//! Abstraction of a virtual machine identifier.
//!
//! Supports both names or uuids.

use std::fmt;
use std::str::FromStr;

use crate::err::Error;


/// Identify a virtual machine.
///
/// The identifier can be created using the `FromStr` trait which will
/// first attempt to parse the input parameters as an uuid, and fallback to
/// treat it as a name.
///
/// ```
/// use vboxhelper::vmid::VmId;
///
/// // will be treated as an VmId::Uuid
/// let mid1 = "00112233-4455-6677-8899-aabbccddeeff".parse::<VmId>();
/// if let Ok(VmId::Uuid(_)) = mid1 {
/// } else {
///   panic!("Not an UUID!");
/// }
///
/// // will be treated as an VmId::Name
/// let mid2 = "myvim".parse::<VmId>();
/// if let Ok(VmId::Name(_)) = mid2 {
/// } else {
///   panic!("Not a name!");
/// }
/// ```
#[derive(Clone)]
pub enum VmId {
  /// Using a name is more human-friendly than an `Uuid`, but it's not
  /// universally unique.
  Name(String),

  /// The uuid is (supposed to be) univeraslly unique, but a little cumbersome
  /// to memorize or type out.
  Uuid(uuid::Uuid)
}


impl fmt::Display for VmId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &*self {
      VmId::Name(n) => write!(f, "{}", n),
      VmId::Uuid(u) => write!(
        f,
        "{{{}}}",
        u.to_hyphenated()
          .encode_lower(&mut uuid::Uuid::encode_buffer())
          .to_string()
      )
    }
  }
}

impl From<&str> for VmId {
  fn from(s: &str) -> Self {
    VmId::Name(s.to_string())
  }
}

impl FromStr for VmId {
  type Err = Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match uuid::Uuid::parse_str(s) {
      Ok(u) => VmId::Uuid(u),
      Err(_) => VmId::Name(s.to_string())
    })
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
