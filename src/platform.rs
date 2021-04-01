use std::path::{Path, PathBuf};

#[cfg(not(windows))]
pub fn get_cmd<P: AsRef<Path>>(nm: P) -> PathBuf {
  PathBuf::from(nm.as_ref())
}

#[cfg(windows)]
pub fn get_cmd<P: AsRef<Path>>(nm: P) -> PathBuf {
  let mut nm = PathBuf::from(nm.as_ref());
  nm.set_extension("exe");

  let exec = match std::env::var_os("VBOX_MSI_INSTALL_PATH") {
    Some(exec) => Path::new(&exec).join(&nm),
    None => nm
  };

  exec
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
