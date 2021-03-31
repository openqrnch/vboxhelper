#[derive(PartialEq, Eq)]
pub(crate) enum EmptyLine {
  Keep,
  Ignore
}

pub(crate) fn buf_to_strlines(buf: &Vec<u8>, el: EmptyLine) -> Vec<String> {
  let sbuf = std::str::from_utf8(&buf).expect("Buffer not UTF-8");

  let mut out = Vec::new();
  for line in sbuf.split("\n") {
    if line.len() == 0 && el == EmptyLine::Ignore {
      continue;
    }
    out.push(line.to_string());
  }

  out
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
