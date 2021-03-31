use std::env;

use vboxhelper::{self, nics::NICType, VmId};


fn main() {
  let args: Vec<String> = env::args().collect();
  let nm = args[1].clone();

  let vmi =
    vboxhelper::get_vm_info(&VmId::Name(nm)).expect("Unable to get vm info");

  for n in &vmi.nics {
    let t = match &n.nictype {
      NICType::Bridged(b) => {
        format!("bridged:{}", b.adapter)
      }
      NICType::IntNet(i) => {
        format!("intnet:{}", i.name)
      }
    };
    println!(
      "idx:[{}]  mac:[{}]  {}",
      n.idx,
      n.mac.to_string(eui48::MacAddressFormat::HexString),
      t
    );
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
