//! Control the run state of a virtual machine.

use std::process::Command;

use crate::platform;
use crate::{Error, RunContext, VmId};


/// Start a virtual machine by UUID or name.
///
/// `ctx` controls how the virtual machine session is launched.  If it
/// is set to [`RunContext::GUI`] the virtual machine will be launched
/// as a frontend context, which requires the caller to be running in a
/// GUI Desktop session.  If it is set to [`RunContext::Headless`]
pub fn start(id: &VmId, ctx: &RunContext) -> Result<(), Error> {
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  let id = id.to_string();
  cmd.arg("startvm");
  cmd.arg(&id);
  cmd.arg("--type");
  cmd.arg(match ctx {
    RunContext::GUI => "gui",
    RunContext::Headless => "headless"
  });

  let out = cmd.output().expect("Unable to execute VBoxManager");

  if out.status.success() {
    Ok(())
  } else {
    Err(Error::CommandFailed(
      out.status.code(),
      "Unable to start command.".to_string()
    ))
  }
}


/// Terminate a virtual machine by UUID or name.
///
/// Killing a virtual machine is normally not a good idea, but it can be
/// useful if the virtual machine is anyway going to be reinstalled or
/// restored to a snapshot.
pub fn kill(id: &VmId) -> Result<(), Error> {
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("controlvm");
  let id = id.to_string();
  cmd.arg(&id);
  cmd.arg("poweroff");

  let out = cmd.output().expect("Unable to execute VBoxManager");
  if out.status.success() {
    Ok(())
  } else {
    Err(Error::CommandFailed(
      out.status.code(),
      "Unable to start command.".to_string()
    ))
  }
}


/// Reset a virtual machine.
pub fn reset(id: &VmId) -> Result<(), Error> {
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("controlvm");
  let id = id.to_string();
  cmd.arg(&id);
  cmd.arg("reset");

  let out = cmd.output().expect("Unable to execute VBoxManager");

  if out.status.success() {
    Ok(())
  } else {
    Err(Error::CommandFailed(
      out.status.code(),
      "Unable to start command.".to_string()
    ))
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
