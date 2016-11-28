use result::{Error, Result};
use std::io::Read;

mod ops;
mod vm;

pub use self::vm::zvm::ZMachine;
