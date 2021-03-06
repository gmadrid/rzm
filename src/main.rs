#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate rzm;

mod args;

use args::Args;
use rzm::{Error, Result, ZMachine};
use std::fs::File;

fn real_main() -> Result<()> {
  let args = Args::parse()?;
  let path = args.zfile();
  let f = File::open(path)?;
  let mut zmachine = ZMachine::from_reader(f)?;
  zmachine.run(args.start_pc()?)?;
  Ok(())
}

fn main() {
  env_logger::init().unwrap();

  // A shell that calls a "real main" function and reports errors.
  // A convenience so that I can try!() inside the "main" function.
  // TODO: make this return non-zero on failure.
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
