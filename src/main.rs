extern crate byteorder;
#[macro_use]
extern crate clap;

mod args;
mod result;
mod zmachine;

use args::Args;
use result::{Error, Result};
use std::fs::File;
use zmachine::ZMachine;

fn real_main() -> Result<()> {
  let args = try!(Args::parse());
  let path = args.zfile();
  let f = try!(File::open(path));

  let mut zmachine = try!(ZMachine::from_reader(f));
  try!(zmachine.run());

  Ok(())
}

fn main() {
  // A shell that calls a "real main" function and reports errors.
  // A convenience so that I can try!() inside the "main" function.
  match real_main() {
    Ok(_) => (),
    Err(err) => {
      match err {
        // Clap gets special attention. ('-h' for example is better handled by clap::Error::exit())
        Error::Clap(ce) => clap::Error::exit(&ce),
        _ => println!("{:?}", err),
      }
    }
  }
}
