use std::env;

use vboxhelper::{self, VmId};

fn main() {
  let args: Vec<String> = env::args().collect();
  let nm = args[1].clone();

  let map = vboxhelper::get_vm_info_map(&VmId::Name(nm))
    .expect("Unable to get VM list");

  let mut keys = Vec::new();

  // let klen = (&map).map(|(key, _)| key.len()).max();
  // let kline = map.keys().map(String::len).max();

  let mut klen = 0;
  for (k, _) in &map {
    if k.len() > klen {
      klen = k.len();
    }
    keys.push(k.clone());
  }

  keys.sort();

  for k in keys {
    let v = map.get(&k).unwrap();
    println!("{:>width$}  {}", k, v, width = klen)
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
