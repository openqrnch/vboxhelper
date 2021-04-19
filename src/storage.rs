use std::borrow::Borrow;
use std::path::Path;
use std::process::Command;

use crate::platform;
use crate::utils;
use crate::Error;
use crate::VmId;

pub enum Type {
  DvdDrive
}

pub struct IdeInfo {
  pub port: u8,
  pub device: u8,
  pub stype: Type
}

pub enum Info {
  IDE(IdeInfo)
}


/// Attach a medium to a storage controller.
pub fn attach<V, P: AsRef<Path>>(
  vid: V,
  info: Info,
  medium: P
) -> Result<(), Error>
where
  V: Borrow<VmId>
{
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  //VBoxManage storageattach $VM --storagectl "IDE" --port 1 --device 0 \
  //  --type dvddrive --medium /usr/share/virtualbox/VBoxGuestAdditions.iso

  cmd.arg("storageattach");
  cmd.arg(vid.borrow().to_string());
  match info {
    Info::IDE(info) => {
      cmd.arg("--storagectl");
      cmd.arg("IDE");
      cmd.arg("--port");
      cmd.arg(info.port.to_string());
      cmd.arg("--device");
      cmd.arg(info.device.to_string());
      cmd.arg("--type");
      match info.stype {
        Type::DvdDrive => {
          cmd.arg("dvddrive");
        }
      }
      cmd.arg("--medium");
      cmd.arg(medium.as_ref());
    }
  }

  utils::exec(cmd)?;

  Ok(())
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
