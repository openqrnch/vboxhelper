//! This crate is probably not what you hoped it was -- in fact, it's probably
//! exactly what you feared.  Rather than integrate against VirtualBox's COM
//! interfaces it will call the command line tools and parse their outputs.
//!
//! Perhaps not surprisingly, this crate originally began as a bash script and
//! slowly morphed into what it is today.
//!
//! # Examples
//!
//! Terminate a virtual machine named _myvm_ and revert it to a snapshot named
//! _mysnap_.
//!
//! ```no_run
//! use std::time::Duration;
//! use vboxhelper::*;
//!
//! let vm = "myvm".parse::<VmId>().unwrap();
//! controlvm::kill(&vm).unwrap();
//!
//! let ten_seconds = Duration::new(10, 0);
//! wait_for_croak(&vm, Some((ten_seconds, TimeoutAction::Error)));
//!
//! // revert to a snapshot
//! let snap = "mysnap".parse::<snapshot::SnapshotId>().unwrap();
//! snapshot::restore(&vm, Some(&snap)).unwrap();
//! ```
//!
//! # VirtualBox Versions
//! This crate will generally attempt to track the latest version of
//! VirtualBox.

mod platform;
mod strutils;
mod utils;

pub mod controlvm;
pub mod err;
pub mod nics;
pub mod snapshot;
pub mod vmid;

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use regex::Regex;

pub use err::Error;

use strutils::{buf_to_strlines, EmptyLine};

pub use vmid::VmId;


/// In what context a virtual machine is run.
pub enum RunContext {
  /// The virtual machine will run as a application on a Desktop GUI.
  GUI,

  /// The virtual machine will run as a background process.
  Headless
}


pub enum TimeoutAction {
  /// If the operation times out, then return an error.
  Error,

  /// If the operation times out, then kill the virtual machine
  Kill
}


pub fn have_vm(id: &VmId) -> Result<bool, Error> {
  let lst = get_vm_list()?;

  for (name, uuid) in lst {
    match id {
      VmId::Name(nm) => {
        if name == *nm {
          return Ok(true);
        }
      }
      VmId::Uuid(u) => {
        if uuid == *u {
          return Ok(true);
        }
      }
    }
  }
  Ok(false)
}


pub fn get_vm_list() -> Result<Vec<(String, uuid::Uuid)>, Error> {
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));
  cmd.args(&["list", "vms"]);

  let output = match cmd.output() {
    Ok(out) => out,
    Err(_) => {
      return Err(Error::FailedToExecute(format!("{:?}", cmd)));
    }
  };

  if !output.status.success() {
    return Err(Error::CommandFailed(format!("{:?}", cmd), output));
  }

  let lines = buf_to_strlines(&output.stdout, EmptyLine::Ignore);

  let mut out = Vec::new();

  for line in lines {
    // Make sure first character is '"'
    match line.find('"') {
      Some(idx) => {
        if idx != 0 {
          continue;
        }
      }
      None => continue
    }

    // Find last '"'
    let idx = match line.rfind('"') {
      Some(idx) => {
        if idx == 0 {
          continue;
        }
        idx
      }
      None => continue
    };


    let idx_ub = match line.rfind('{') {
      Some(idx) => {
        if idx == 0 {
          continue;
        }
        idx
      }
      None => continue
    };

    let idx_ue = match line.rfind('}') {
      Some(idx) => {
        if idx == 0 {
          continue;
        }
        idx
      }
      None => continue
    };

    let name = &line[1..idx];
    let uuidstr = &line[(idx_ub + 1)..idx_ue];
    let u = match uuid::Uuid::parse_str(uuidstr) {
      Ok(u) => u,
      Err(_) => continue
    };
    out.push((name.to_string(), u));
  }

  Ok(out)
}


/// Get information about a virtual machine as a map.
pub fn get_vm_info_map(id: &VmId) -> Result<HashMap<String, String>, Error> {
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));
  cmd.arg("showvminfo");
  let id_str = id.to_string();
  cmd.arg(&id_str);
  cmd.arg("--machinereadable");

  let output = cmd.output().expect("Failed to execute VBoxManage");

  let lines = strutils::buf_to_strlines(&output.stdout, EmptyLine::Ignore);

  let mut map = HashMap::new();

  // multiline
  let re_ml1 = Regex::new(r#"^(?P<key>[^"=]+)="(?P<val>[^"=]*)"$"#).unwrap();
  let re_ml1 = Regex::new(r#"^"(?P<key>[^"=]+)"="(?P<val>[^"=]*)"$"#).unwrap();

  // Capture foo="bar" -> foo=bar
  // This appears to be most common.
  let re1 = Regex::new(r#"^(?P<key>[^"=]+)="(?P<val>[^"=]*)"$"#).unwrap();

  // Capture "foo"="bar" -> foo=bar
  let re2 = Regex::new(r#"^"(?P<key>[^"=]+)"="(?P<val>[^"=]*)"$"#).unwrap();

  // foo=bar -> foo=bar
  let re3 = Regex::new(r#"^(?P<key>[^"=]+)=(?P<val>[^"=]*)$"#).unwrap();

  //let re = Regex::new(r#"^"?(?P<key>[^"=]+)"?="?(?P<val>[^"=]*)"?$"#).
  // unwrap();


  // ToDo: Handle multiline entires, like descriptions
  let mut lines = lines.iter();
  while let Some(line) = lines.next() {
    //println!("line: {}", line);

    let line = line.trim_end();
    let cap = if let Some(cap) = re1.captures(&line) {
      Some(cap)
    } else if let Some(cap) = re2.captures(&line) {
      Some(cap)
    } else if let Some(cap) = re3.captures(&line) {
      Some(cap)
    } else {
      println!("Ignored line: {}", line);
      None
    };

    if let Some(cap) = cap {
      map.insert(cap[1].to_string(), cap[2].to_string());
    }
  }

  Ok(map)
}


#[derive(PartialEq, Eq)]
/// VirtualBox virtual machine states.
pub enum VmState {
  /// This isn't actually a VirtualBox virtual machine state; it's used as a
  /// placeholder if an unknown state is encountered.
  Unknown,

  /// The virtual machine is powered off.
  PowerOff,

  /// The virtual machine is currently starting up.
  Starting,

  /// The virtual machine is currently up and running.
  Running,

  /// The virtual machine is currently paused.
  Paused,

  /// The virtual machine is currently shutting down.
  Stopping
}

impl From<&str> for VmState {
  fn from(s: &str) -> Self {
    match s {
      "poweroff" => VmState::PowerOff,
      "starting" => VmState::Starting,
      "running" => VmState::Running,
      "paused" => VmState::Paused,
      "stopping" => VmState::Stopping,
      _ => VmState::Unknown
    }
  }
}

impl From<&String> for VmState {
  fn from(s: &String) -> Self {
    match s.as_ref() {
      "poweroff" => VmState::PowerOff,
      "starting" => VmState::Starting,
      "running" => VmState::Running,
      "paused" => VmState::Paused,
      "stopping" => VmState::Stopping,
      _ => VmState::Unknown
    }
  }
}


/// A structured representation of a virtual machine's state and configuration.
pub struct VmInfo {
  pub shares_map: HashMap<String, PathBuf>,
  pub shares_list: Vec<(String, PathBuf)>,
  pub state: VmState,
  pub snapshots: Option<snapshot::Snapshots>,
  pub nics: Vec<nics::NICInfo>
}


/// Get structured information about a virtual machine.
pub fn get_vm_info(id: &VmId) -> Result<VmInfo, Error> {
  let map = get_vm_info_map(id)?;

  let mut shares_list = Vec::new();
  let mut shares_map = HashMap::new();

  //
  // Parse shares
  //
  let mut idx = 1;
  loop {
    let name_key = format!("SharedFolderNameMachineMapping{}", idx);
    let path_key = format!("SharedFolderPathMachineMapping{}", idx);

    let name = match map.get(&name_key) {
      Some(nm) => nm.clone(),
      None => break
    };
    let pathname = match map.get(&path_key) {
      Some(pn) => PathBuf::from(pn),
      None => break
    };

    shares_map.insert(name.clone(), pathname.clone());
    shares_list.push((name, pathname));

    idx += 1;
  }

  //
  // Get VM State
  //
  let state = match map.get("VMState") {
    Some(s) => VmState::from(s),
    None => VmState::Unknown
  };

  //
  // Parse snapshots
  //
  let snaps = snapshot::get_from_map(&map)?;

  //
  // Parse NICs
  //
  let nics = nics::get_from_map(&map)?;

  Ok(VmInfo {
    state,
    shares_map,
    shares_list,
    snapshots: snaps,
    nics
  })
}


/// Check whether a virtual machine is currently in a certain state.
pub fn is_vm_state(id: &VmId, state: VmState) -> Result<bool, Error> {
  let vmi = get_vm_info(id)?;
  Ok(vmi.state == state)
}


/// Wait for a virtual machine to self-terminate.
///
/// The caller can choose to pass a timeout and what action should be taken if
/// the operation times out.  If the timeout occurs the caller can choose
/// whether to return a timeout error or kill the virtual machine.
///
/// ```no_run
/// use std::time::Duration;
/// use vboxhelper::{TimeoutAction, wait_for_croak, VmId};
/// fn impatient() {
///   let twenty_seconds = Duration::new(20, 0);
///   let vmid = VmId::from("myvm");
///   wait_for_croak(&vmid, Some((twenty_seconds, TimeoutAction::Kill)));
/// }
/// ```
///
/// This function polls `is_vm_state()` which calls `get_vm_info()`.  A very
/// sad state of affairs.  :(
pub fn wait_for_croak(
  id: &VmId,
  timeout: Option<(Duration, TimeoutAction)>
) -> Result<(), Error> {
  let start = Instant::now();
  loop {
    let poweroff = is_vm_state(id, VmState::PowerOff)?;
    if poweroff {
      break;
    }
    if let Some((ref max_dur, ref action)) = timeout {
      let duration = start.elapsed();
      if duration > *max_dur {
        match action {
          TimeoutAction::Error => return Err(Error::Timeout),
          TimeoutAction::Kill => {
            controlvm::kill(id)?;

            // ToDo: Give it some time to croak.  If it doesn't, then return
            //       an "uncroakable vm" error.
            break;
          }
        }
      }
    }

    // Why 11?  Because it's more than 10, and it's a prime.  I don't know why
    // 11 is a prime -- ask the universe.
    let eleven_secs = Duration::from_secs(11);
    thread::sleep(eleven_secs);
  }
  Ok(())
}


/*
fn foo() {
  let _map = get_vm_info_map("hello").unwrap();
}
*/

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
