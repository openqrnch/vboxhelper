use std::collections::HashMap;

use crate::err::Error;


#[derive(Debug)]
pub struct BridgedNIC {
  pub adapter: String
}

#[derive(Debug)]
pub struct IntNetNIC {
  pub name: String
}

#[derive(Debug)]
pub enum NICType {
  Bridged(BridgedNIC),
  IntNet(IntNetNIC)
}

#[derive(Debug)]
pub struct NICInfo {
  pub idx: u8,
  pub nictype: NICType,
  pub mac: eui48::MacAddress
}


pub fn get_from_map(
  map: &HashMap<String, String>
) -> Result<Vec<NICInfo>, Error> {
  let mut nics: Vec<NICInfo> = Vec::new();

  for idx in 1..=8 {
    let key = format!("nic{}", idx);
    if let Some(v) = map.get(&key) {
      if v == "none" {
        continue;
      }
      let nictype = match v.as_ref() {
        "none" => continue,
        "bridged" => {
          let key = format!("bridgeadapter{}", idx);
          let adapter = match map.get(&key) {
            Some(adapter) => adapter,
            None => {
              // missing critical information..
              continue;
            }
          };
          NICType::Bridged(BridgedNIC {
            adapter: adapter.to_string()
          })
        }
        "intnet" => {
          let key = format!("intnet{}", idx);
          let name = match map.get(&key) {
            Some(name) => name,
            None => {
              // missing critical information..
              continue;
            }
          };
          NICType::IntNet(IntNetNIC {
            name: name.to_string()
          })
        }
        _ => {
          println!("unrecognized nic type: {}", v);
          continue;
        }
      };

      let key = format!("macaddress{}", idx);
      let mac = match map.get(&key) {
        Some(addr) => eui48::MacAddress::parse_str(addr)?,
        None => continue
      };

      nics.push(NICInfo { idx, nictype, mac });
    }
  }

  Ok(nics)
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
