extern crate byteorder;
extern crate clap;
extern crate rand;

mod result;
mod zmachine;

pub use result::{Error, Result};
pub use zmachine::ZMachine;
