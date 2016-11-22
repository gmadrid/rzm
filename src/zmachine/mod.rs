use result::{Error, Result};
use std::io::Read;

mod ops;
mod object_table;
mod vm;

pub use self::vm::zvm::ZMachine;
