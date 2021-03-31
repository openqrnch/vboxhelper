use vboxhelper;

fn main() {
  let lst = vboxhelper::get_vm_list().expect("Unable to get VM list");

  let mut max_len = 0;
  for (nm, _) in &lst {
    if nm.len() > max_len {
      max_len = nm.len();
    }
  }

  for (nm, uuid) in &lst {
    println!("{:width$}  {}", nm, uuid, width = max_len);
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
