use clap;
use std::io;
use std::result;
use zmachine::opcodes::Operands;

#[derive(Debug)]
pub enum Error {
  Clap(clap::Error),
  IO(io::Error),

  BadOperands(String, Operands),
  CouldNotReadHeader,
  UnknownOpcode(u8, usize),
  ZFileTooShort,
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
