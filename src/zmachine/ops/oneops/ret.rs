use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};
use zmachine::ops::ret_value;

pub fn ret_0x0b<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  let value = operand.value(runner);
  println!("ret {:?}: {:x}", operand, value);
  ret_value(runner, value)
}
