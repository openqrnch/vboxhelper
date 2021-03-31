use std::env;

use vboxhelper::{self, snapshot, VmId};

fn main() {
  let args: Vec<String> = env::args().collect();
  let nm = args[1].clone();

  let map =
    snapshot::map(&VmId::Name(nm)).expect("Unable to get snapshot map");

  for (k, v) in map.iter() {
    println!("{}  {}", k, v);
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
