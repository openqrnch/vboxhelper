//! Control the run state of a virtual machine.

use std::borrow::Borrow;
use std::process::Command;

use crate::platform;
use crate::{Error, Headless, RunContext, VmId};


/// Start a virtual machine by UUID or name.
///
/// `ctx` controls how the virtual machine session is launched.  If it
/// is set to [`RunContext::GUI`] the virtual machine will be launched
/// as a GUI frontend context, which requires the caller to be running in a
/// GUI Desktop session.  If it is set to [`RunContext::Headless`] the VM
/// will run without a frontend GUI.
pub fn start<V, R>(vid: V, ctx: R) -> Result<(), Error>
where
  V: Borrow<VmId>,
  R: Borrow<RunContext>
{
  let mut cmd = match ctx.borrow() {
    RunContext::GUI => {
      let mut cmd = Command::new(platform::get_cmd("VBoxManage"));
      cmd.arg("startvm");
      cmd.arg(vid.borrow().to_string());
      cmd.arg("--type");
      cmd.arg("gui");

      cmd
    }
    RunContext::Headless(Headless::Detached) => {
      let mut cmd = Command::new(platform::get_cmd("VBoxManage"));
      cmd.arg("startvm");
      cmd.arg(vid.borrow().to_string());
      cmd.arg("--type");
      cmd.arg("headless");

      cmd
    }
    RunContext::Headless(Headless::Blocking) => {
      let mut cmd = Command::new(platform::get_cmd("VBoxHeadless"));
      cmd.arg("--startvm");
      cmd.arg(vid.borrow().to_string());

      cmd
    }
  };

  let out = match cmd.output() {
    Ok(out) => out,
    Err(_) => {
      return Err(Error::FailedToExecute(format!("{:?}", cmd)));
    }
  };

  if out.status.success() {
    Ok(())
  } else {
    let s = format!("{:?}", cmd);
    Err(Error::CommandFailed(s, out))
  }
}


/// Terminate a virtual machine by UUID or name.
///
/// Killing a virtual machine is normally not a good idea, but it can be
/// useful if the virtual machine is anyway going to be reinstalled or
/// restored to a snapshot.
pub fn kill<V>(vid: V) -> Result<(), Error>
where
  V: Borrow<VmId>
{
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("controlvm");
  //let id = id.to_string();
  cmd.arg(vid.borrow().to_string());
  cmd.arg("poweroff");

  let out = match cmd.output() {
    Ok(out) => out,
    Err(_) => {
      return Err(Error::FailedToExecute(format!("{:?}", cmd)));
    }
  };

  if out.status.success() {
    Ok(())
  } else {
    Err(Error::CommandFailed(format!("{:?}", cmd), out))
  }
}


/// Reset a virtual machine.
pub fn reset<V>(vid: V) -> Result<(), Error>
where
  V: Borrow<VmId>
{
  let mut cmd = Command::new(platform::get_cmd("VBoxManage"));

  cmd.arg("controlvm");
  cmd.arg(vid.borrow().to_string());
  cmd.arg("reset");

  let out = match cmd.output() {
    Ok(out) => out,
    Err(_) => {
      let s = format!("{:?}", cmd);
      return Err(Error::FailedToExecute(s));
    }
  };

  if out.status.success() {
    Ok(())
  } else {
    Err(Error::CommandFailed(format!("{:?}", cmd), out))
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
