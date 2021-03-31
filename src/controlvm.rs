//! Control the run state of a virtual machine.

use std::process::Command;

use crate::{Error, RunContext, VmId};

/// Start a virtual machine by UUID or name.
///
/// `ctx` controls how the virtual machine session is launched.  If it
/// is set to [`RunContext::GUI`] the virtual machine will be launched
/// as a frontend context, which requires the caller to be running in a
/// GUI Desktop session.  If it is set to [`RunContext::Headless`]
pub fn start(id: &VmId, ctx: &RunContext) -> Result<(), Error> {
  let mut args = Vec::new();

  let id = id.to_string();
  args.push("startvm");
  args.push(&id);
  args.push("--type");
  args.push(match ctx {
    RunContext::GUI => "gui",
    RunContext::Headless => "headless"
  });

  let out = Command::new("VBoxManage")
    .args(args)
    .output()
    .expect("Unable to execute VBoxManager");

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
  let mut args = Vec::new();

  let id = id.to_string();
  args.push("controlvm");
  args.push(&id);
  args.push("poweroff");

  let out = Command::new("VBoxManage")
    .args(args)
    .output()
    .expect("Unable to execute VBoxManager");

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
  let mut args = Vec::new();

  let id = id.to_string();
  args.push("controlvm");
  args.push(&id);
  args.push("reset");

  let out = Command::new("VBoxManage")
    .args(args)
    .output()
    .expect("Unable to execute VBoxManager");

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
