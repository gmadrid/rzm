use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operand, Operands};
use zmachine::ops::twoops::je_0x01;

pub fn storew_0x01<T>(runner: &mut T, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  if let Operands::Var(arr) = operands {
    if arr[0..2].iter().any(|o| *o == Operand::Omitted) {
      return Err(Error::BadOperands("3 operands required".to_string(), Operands::Var(arr)));
    }
    let array = arr[0].value(runner);
    let word_index = arr[1].value(runner);
    let val = arr[2].value(runner);
    println!("THAT VAL: {:x}/{:x}/{:x}", array, word_index, val);
  } else {
    return Err(Error::BadOperands("3 operands required".to_string(), operands));
  }
  Ok(())
}
