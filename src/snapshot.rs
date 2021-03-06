//! Manage virtual machine snapshots.

use std::borrow::Borrow;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::process::Command;
use std::str::FromStr;

use regex::Regex;

use crate::platform;
use crate::strutils::{buf_to_strlines, EmptyLine};
use crate::utils;
use crate::VmId;

use crate::Error;

pub enum SnapshotId {
  Name(String),
  Uuid(uuid::Uuid)
}

impl ToString for SnapshotId {
  fn to_string(&self) -> String {
    match &*self {
      SnapshotId::Name(s) => s.clone(),
      SnapshotId::Uuid(u) => u
        .to_hyphenated()
        .encode_lower(&mut uuid::Uuid::encode_buffer())
        .to_string()
    }
  }
}

impl From<&str> for SnapshotId {
  fn from(s: &str) -> Self {
    SnapshotId::Name(s.to_string())
  }
}

impl FromStr for SnapshotId {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match uuid::Uuid::parse_str(s) {
      Ok(u) => SnapshotId::Uuid(u),
      Err(_) => SnapshotId::Name(s.to_string())
    })
  }
}


#[derive(Debug)]
pub struct Snapshot {
  pub name: String,
  pub uuid: uuid::Uuid,
  pub desc: Vec<String>,
  pub children: Vec<uuid::Uuid>
}

impl Hash for Snapshot {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.uuid.hash(state);
  }
}

impl PartialEq for Snapshot {
  fn eq(&self, other: &Self) -> bool {
    self.uuid == other.uuid
  }
}

impl Eq for Snapshot {}


/// Get a HashMap of all snapshots.
pub fn map<V>(vid: V) -> Result<HashMap<String, String>, Error>
where
  V: Borrow<VmId>
{
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("snapshot");
  cmd.arg(vid.borrow().to_string());
  cmd.arg("list");
  cmd.arg("--machinereadable");

  let (stdout, _) = utils::exec(cmd)?;

  let lines = buf_to_strlines(&stdout, EmptyLine::Ignore);

  let mut map = HashMap::new();

  // Ugly hack -- refine as we go along
  let re = Regex::new(r#"^"?(?P<key>[^"=]+)"?="?(?P<val>[^"=]*)"?$"#).unwrap();

  for line in lines {
    //println!("line: {}", line);
    let mut chit = line.chars();
    let ch = chit.next().unwrap();
    if ch == '-' {
      // Ignore descriptions for now
      continue;
    }

    let cap = match re.captures(&line) {
      Some(cap) => cap,
      None => continue
    };

    map.insert(cap[1].to_string(), cap[2].to_string());
  }

  Ok(map)
}


pub struct Snapshots {
  pub map: HashMap<uuid::Uuid, Snapshot>,
  pub root: uuid::Uuid,
  pub current: uuid::Uuid
}

impl Snapshots {
  fn get<S>(&self, sid: S) -> Vec<&Snapshot>
  where
    S: Borrow<SnapshotId>
  {
    match sid.borrow() {
      SnapshotId::Name(nm) => self.get_by_name(nm),
      SnapshotId::Uuid(u) => match self.get_by_uuid(u) {
        Some(u) => vec![u],
        None => Vec::new()
      }
    }
  }

  pub fn get_root(&self) -> Option<&Snapshot> {
    self.map.get(&self.root)
  }
  pub fn get_current(&self) -> Option<&Snapshot> {
    self.map.get(&self.current)
  }

  pub fn get_by_uuid(&self, uuid: &uuid::Uuid) -> Option<&Snapshot> {
    self.map.get(uuid)
  }

  pub fn get_by_name<N>(&self, name: N) -> Vec<&Snapshot>
  where
    N: AsRef<str>
  {
    let mut out = Vec::new();
    for (_, snap) in &self.map {
      if snap.name.as_str() == name.as_ref() {
        out.push(snap);
      }
    }
    out
  }

  pub fn get_unique_by_name(&self, name: &str) -> Result<&Snapshot, Error> {
    let snaplist = self.get_by_name(name);
    match snaplist.len() {
      0 => {
        let s = format!("The VM has no snapshot named '{}'", name);
        return Err(Error::MissingData(s));
      }
      1 => Ok(snaplist[0]),
      _ => {
        let s = format!("The VM has multiple snapshots named '{}'", name);
        return Err(Error::Ambiguous(s));
      }
    }
  }
}


/// Get a structured representation of all snapshots for a virtual machine.
pub fn get<V>(vid: V) -> Result<Option<Snapshots>, Error>
where
  V: Borrow<VmId>
{
  // Get snapshots as a HashMap
  let map = map(vid)?;

  // Convert snapshots HashMap into a structure
  get_from_map(&map)
}


/// Convert a HashMap of snapshots (typically aquired using [`map()`]) to a
/// structured representation of the snapshots tree.
pub fn get_from_map(
  map: &HashMap<String, String>
) -> Result<Option<Snapshots>, Error> {
  // Keep track of snapshot tree nodes while iterating.
  // The HashSet isn't used here because it does not allow nodes to be edited.
  let mut snapmap = HashMap::new();

  let mut root_uuid: Option<uuid::Uuid> = None;
  let mut current_uuid: Option<uuid::Uuid> = None;

  let mut q = VecDeque::new();

  // Get root snapshot
  match (map.get("SnapshotName"), map.get("SnapshotUUID")) {
    (Some(_), Some(uid)) => {
      // Push "branch" (empty string, since it's the root node) on to stack
      q.push_back("".to_string());

      root_uuid = match uuid::Uuid::parse_str(uid) {
        Ok(u) => Some(u),
        Err(_) => {
          let s = format!("Unable to parse root UUID '{}'", uid);
          return Err(Error::BadFormat(s));
        }
      };
    }
    _ => {
      // No results
      return Ok(None);
    }
  }

  while !q.is_empty() {
    let curbranch = q.pop_back().unwrap();

    let uuid_key = format!("SnapshotUUID{}", curbranch);
    let name_key = format!("SnapshotName{}", curbranch);

    let nm = map.get(&name_key).unwrap();
    let uid = map.get(&uuid_key).unwrap();

    let u = match uuid::Uuid::parse_str(uid) {
      Ok(u) => u,
      Err(_) => {
        let s = format!("Unable to parse UUID '{}' for '{}'", uid, uuid_key);
        return Err(Error::BadFormat(s));
      }
    };

    snapmap.insert(
      u,
      Snapshot {
        name: nm.clone(),
        uuid: u,
        desc: Vec::new(),
        children: Vec::new()
      }
    );

    // Generate a -X-Y-Z branch name, stick it to a "SnapshotUUID" string and
    // see if the combined name exists in the map.  If it does, then
    for i in 1..usize::MAX {
      let branch = format!("{}-{}", curbranch, i);

      let key = format!("SnapshotUUID{}", branch);

      // If this key exists in the map, then add it to the queue
      if let Some(v) = map.get(&key) {
        // Get UUID of current child node
        let cuid = match uuid::Uuid::parse_str(v) {
          Ok(u) => u,
          Err(_) => {
            let s = format!(
              "Unable to parse UUID '{}' for 'SnapshotUUID{}'",
              uid, curbranch
            );
            return Err(Error::BadFormat(s));
          }
        };

        // Add child UUID to its parent
        if let Some(n) = snapmap.get_mut(&u) {
          n.children.push(cuid);
        }

        // Push this child on to the processing queue for further processing
        q.push_back(branch);
      } else {
        // No more children -- break out of index iteration
        break;
      }
    }
  }

  if let Some(us) = map.get("CurrentSnapshotUUID") {
    current_uuid = Some(match uuid::Uuid::parse_str(us) {
      Ok(u) => u,
      Err(_) => {
        let s = format!("Unable to parse current UUID '{}'", us);
        return Err(Error::BadFormat(s));
      }
    });
  } else {
    return Err(Error::MissingData(
      "Can't find expected field 'CurrentSnapshotUUID".to_string()
    ));
  }

  let snaps = Snapshots {
    map: snapmap,
    root: match root_uuid {
      Some(u) => u,
      None => {
        // Ok to panic since we already know we have it
        panic!("No root UUID");
      }
    },
    current: match current_uuid {
      Some(u) => u,
      None => {
        // Ok to panic since we know we have it
        panic!("No current UUID");
      }
    }
  };

  Ok(Some(snaps))
}


/// Take a new snapshot at current vm state.
pub fn take<V, N>(vid: V, nm: N) -> Result<(), Error>
where
  V: Borrow<VmId>,
  N: AsRef<str>
{
  // VBoxManage snapshot <vid> take <nm>

  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));
  cmd.arg("snapshot");
  cmd.arg(vid.borrow().to_string());
  cmd.arg("take");
  cmd.arg(nm.as_ref());

  utils::exec(cmd)?;

  Ok(())
}


/// Returns `Ok(true)` if the virtual machine `vid` has one or more snapshots
/// named `name`.
pub fn have_name<V, N>(vid: V, name: N) -> Result<bool, Error>
where
  V: Borrow<VmId>,
  N: AsRef<str>
{
  if let Some(snaps) = get(vid)? {
    let snaplist = snaps.get_by_name(name);
    if snaplist.is_empty() {
      Ok(false)
    } else {
      Ok(true)
    }
  } else {
    Ok(false)
  }
}


/// Returns `Ok(true)` if the virtual machine `vid` has a single snapshot
/// named `name`.  Returns `Err(Missing)` if there are no snapshots with that
/// name.  Returns `Err(Error::Ambiguous)` if there's more than one.
pub fn check_unique_name<V, N>(vid: V, name: N) -> Result<(), Error>
where
  V: Borrow<VmId>,
  N: AsRef<str>
{
  if let Some(snaps) = get(vid.borrow())? {
    let snaplist = snaps.get_by_name(name.borrow());
    if snaplist.len() == 1 {
      return Ok(());
    } else if snaplist.len() > 1 {
      let s = format!(
        "Virtual machine '{}' has multiple snapshots named '{}'",
        vid.borrow().to_string(),
        name.as_ref()
      );
      return Err(Error::Ambiguous(s));
    }
  }

  let s = format!(
    "Virtual machine '{}' has no snapshot named '{}'",
    vid.borrow().to_string(),
    name.as_ref()
  );
  Err(Error::Missing(s))
}


/// Rename a snapshot
pub fn rename<V, S, N>(vid: V, sid: S, newname: N) -> Result<(), Error>
where
  V: Borrow<VmId>,
  S: Borrow<SnapshotId>,
  N: AsRef<str>
{
  // VBoxManage snapshot <vid> edit <sid> --name=<newname>

  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));
  cmd.arg("snapshot");
  cmd.arg(vid.borrow().to_string());
  cmd.arg("edit");
  cmd.arg(sid.borrow().to_string());
  cmd.arg(format!("--name={}", newname.as_ref()));

  utils::exec(cmd)?;

  Ok(())
}


/// Restore a virtual machine to a snapshot.
///
/// If `snap_id` is `None` the "current" snapshot is restored.  Otherwise
/// `snap_id` should be a `SnapshotId` which identified a snapshot to restore.
pub fn restore<V, S>(vid: V, snap_id: Option<S>) -> Result<(), Error>
where
  V: Borrow<VmId>,
  S: Borrow<SnapshotId>
{
  if let Some(ref snap_id) = snap_id {
    if let SnapshotId::Name(nm) = snap_id.borrow() {
      let snaps = get(vid.borrow())?;
      if let Some(snaps) = snaps {
        let snaplist = snaps.get_by_name(nm);
        if snaplist.len() > 1 {
          let s = format!(
            "The VM '{}' has multiple snapshots named '{}'",
            vid.borrow().to_string(),
            nm
          );
          return Err(Error::Ambiguous(s));
        }
      } else {
        // Here we should probably abort, because if no snapshots are found
        // then there's little point in going on.
      }
    }
  }

  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("snapshot".to_string());
  cmd.arg(vid.borrow().to_string());
  if let Some(snap_id) = snap_id {
    cmd.arg("restore".to_string());
    cmd.arg(snap_id.borrow().to_string());
  } else {
    cmd.arg("restorecurrent".to_string());
  }

  utils::exec(cmd)?;

  Ok(())
}


/// Delete a snapshot.
///
/// Croaks if the snapshot does not exist.
pub fn delete<V, S>(vid: V, sid: S) -> Result<(), Error>
where
  V: Borrow<VmId>,
  S: Borrow<SnapshotId>
{
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("snapshot");
  cmd.arg(vid.borrow().to_string());
  cmd.arg("delete");
  cmd.arg(sid.borrow().to_string());

  utils::exec(cmd)?;

  Ok(())
}


/// Just like `delete()` but checks if the snapshot exists first.
pub fn delete_if_exists<V, S>(vid: V, sid: S) -> Result<(), Error>
where
  V: Borrow<VmId>,
  S: Borrow<SnapshotId>
{
  let snaps = get(vid.borrow())?;

  if let Some(snaps) = snaps {
    let snaplist = snaps.get(sid.borrow());

    if snaplist.is_empty() {
      return Ok(());
    }

    // Seems like snapshot exists -- attempt to delete it
    delete(vid, sid)?;
  }

  Ok(())
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
