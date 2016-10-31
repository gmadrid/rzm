use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};
use zmachine::ops::twoops::je_0x01;

pub fn jz_0x00<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  je_0x01(runner, operand, Operand::SmallConstant(0))
}

pub fn jump_0x0c<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  let value = operand.value(runner) as i16 as isize;
  let current_pc = runner.current_pc();
  let new_pc = ((current_pc as isize) + value) as usize - 2;
  runner.set_current_pc(new_pc);
  Ok(())
}
