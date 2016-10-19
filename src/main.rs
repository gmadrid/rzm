extern crate ansi_term;
#[macro_use]
extern crate clap;

mod args;
mod hunt;
mod result;

use hunt::board::Board;
use result::{Error, Result};

fn real_main() -> Result<()> {
  let _ = try!(args::parse());

  let mut board = Board::alpha();
  board.dump();  // TODO: replace this with a display() method.
  println!("");

  board.north();
  board.dump();
  println!("");

  board.west();
  board.west();
  board.west();
  board.south();
  board.east();
  board.dump();

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
