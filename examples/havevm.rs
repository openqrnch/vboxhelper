use std::env;

use vboxhelper::{self, VmId};

fn main() {
  let args: Vec<String> = env::args().collect();
  let id = args[1].clone();

  let id = id.parse::<VmId>().expect("Unable to parse id");

  let have = vboxhelper::have_vm(&id).expect("Fatal error getting VM status");

  if have {
    println!("Yes, the VM '{}' exists", id);
  } else {
    println!("No, the VM '{}' does not exist", id);
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
