use std::env;

use vboxhelper::{self, VmId};

fn main() {
  let args: Vec<String> = env::args().collect();
  let nm = args[1].clone();

  let vmi =
    vboxhelper::get_vm_info(&VmId::Name(nm)).expect("Unable to get vm info");

  for (name, path) in &vmi.shares_list {
    println!("{}  {}", name, path.display());
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
