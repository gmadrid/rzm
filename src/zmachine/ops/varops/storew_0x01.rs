use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operand};

pub fn storew_0x01<T>(runner: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: OpcodeRunner {
  if operands[0..2].iter().any(|o| *o == Operand::Omitted) {
    panic!("3 operands required: {:?}", operands);
  }
  let array = operands[0].value(runner) as usize;
  let word_index = operands[1].value(runner) as usize;
  let val = operands[2].value(runner);
  println!("storew {:x}/{:x}/{:x} == {:?}",
           array,
           word_index,
           val,
           operands[2]);
  runner.write_memory(array + 2 * word_index, val);

  Ok(())
}
