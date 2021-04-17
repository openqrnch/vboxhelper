use std::process::Command;

use crate::err::Error;


pub(crate) fn exec(mut cmd: Command) -> Result<(Vec<u8>, Vec<u8>), Error> {
  let out = match cmd.output() {
    Ok(out) => out,
    Err(_) => {
      return Err(Error::FailedToExecute(format!("{:?}", cmd)));
    }
  };

  if out.status.success() {
    Ok((out.stdout, out.stderr))
  } else {
    Err(Error::CommandFailed(format!("{:?}", cmd), out))
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
