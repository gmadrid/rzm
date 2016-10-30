use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};

pub fn ret_0x0b<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  let value = operand.value(runner);
  println!("{:?}: {:x}", operand, value);

  let (pc, result_var) = runner.pop_frame(value);
  runner.write_to_variable(result_var, value);
  runner.set_current_pc(pc);

  Ok(())
}
