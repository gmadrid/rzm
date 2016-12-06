use clap;
use std::io;
use std::num;
use std::result;

#[derive(Debug)]
pub enum Error {
  BoardParseError,
  Clap(clap::Error),
  IO(io::Error),

  CouldNotReadHeader,
  ParseIntError(&'static str, num::ParseIntError),
  UnknownOpcode(&'static str, u8, usize),
  ZFileTooShort,

  Quitting,
}

pub type Result<T> = result::Result<T, Error>;

impl From<clap::Error> for Error {
  fn from(err: clap::Error) -> Error {
    Error::Clap(err)
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Error {
    Error::IO(err)
  }
}
