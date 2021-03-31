use std::collections::VecDeque;
use std::env;

use vboxhelper::{self, snapshot, VmId};

fn main() {
  let indent_width = 2;
  let istr = ' '.to_string().repeat(indent_width);

  let args: Vec<String> = env::args().collect();
  let nm = args[1].clone();

  let snaps = snapshot::get(&VmId::Name(nm)).expect("Unable to get snapshots");

  // This works as well
  /*
  let snaps =
    vboxhelper::get_vm_info(&VmId::Name(nm)).expect("Unable to get VM list");
  let snaps = snaps.snapshots;
  */

  let snaps = snaps.expect("No results");

  let root = snaps.get_root().expect("Root not found");

  let mut stack = Vec::new();

  let mut q = VecDeque::new();
  q.push_back(root);
  stack.push(q);

  while !stack.is_empty() {
    let mut q = stack.pop().unwrap();

    while !q.is_empty() {
      let s = q.pop_front().unwrap();

      let level = stack.len();

      let curstr = if snaps.current == s.uuid {
        " (current)"
      } else {
        ""
      };

      println!("{}{} {{{}}}{}", istr.repeat(level), s.name, s.uuid, curstr);

      // If this node has children, process them
      if !s.children.is_empty() {
        let mut children = VecDeque::new();
        for u in &s.children {
          if let Some(snap) = snaps.get_by_uuid(u) {
            children.push_back(snap);
          }
        }

        // Put "current" queue back
        stack.push(q);

        // Put new queue on stack
        stack.push(children);
        break;
      }
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
