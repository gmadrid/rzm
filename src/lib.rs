extern crate byteorder;
extern crate clap;

mod result;
mod zmachine;

pub use result::{Error, Result};
pub use zmachine::ZMachine;
