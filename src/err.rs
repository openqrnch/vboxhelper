use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
  IO(String),
  BadFormat(String),
  FailedToExecute(String),
  CommandFailed(String, std::process::Output),
  MissingData(String),
  Ambiguous(String),
  Missing(String),
  Timeout
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Error::IO(err.to_string())
  }
}

impl From<eui48::ParseError> for Error {
  fn from(err: eui48::ParseError) -> Self {
    Error::BadFormat(err.to_string())
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::IO(s) => write!(f, "I/O error; {}", s),
      Error::BadFormat(s) => write!(f, "Bad format error; {}", s),
      Error::FailedToExecute(s) => {
        write!(f, "Failed to start command; {}", s)
      }
      Error::CommandFailed(s, output) => match output.status.code() {
        Some(code) => {
          write!(f, "Command returned failure; exit status={}; {}", code, s)
        }
        None => write!(f, "Command returned failure; {}", s)
      },
      Error::MissingData(s) => write!(f, "Missing expected data error; {}", s),
      Error::Missing(s) => write!(f, "Unexpectedly missing; {}", s),
      Error::Ambiguous(s) => write!(f, "Ambiguity error; {}", s),
      Error::Timeout => write!(f, "Timeout")
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
