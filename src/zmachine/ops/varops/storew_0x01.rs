use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operand, Operands};

pub fn storew_0x01<T>(runner: &mut T, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  if let Operands::Var(arr) = operands {
    if arr[0..2].iter().any(|o| *o == Operand::Omitted) {
      return Err(Error::BadOperands("3 operands required".to_string(), Operands::Var(arr)));
    }
    let array = arr[0].value(runner) as usize;
    let word_index = arr[1].value(runner) as usize;
    let val = arr[2].value(runner);
    println!("storew {:x}/{:x}/{:x} == {:?}",
             array,
             word_index,
             val,
             arr[2]);
    runner.write_memory(array + 2 * word_index, val);
  } else {
    return Err(Error::BadOperands("3 operands required".to_string(), operands));
  }
  Ok(())
}
