extern crate byteorder;
extern crate clap;
#[macro_use]
extern crate log;
extern crate ncurses;
extern crate rand;

mod result;
mod zmachine;

pub use result::{Error, Result};
pub use zmachine::ZMachine;
