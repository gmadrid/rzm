use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};

mod binop;
mod branch;
mod call;
mod properties;

pub mod oneops;
pub mod twoops;
pub mod varops;

fn ret_value<T>(runner: &mut T, value: u16) -> Result<()>
  where T: OpcodeRunner {
  let (pc, result_var) = runner.pop_frame(value);
  runner.write_to_variable(result_var, value);
  runner.set_current_pc(pc);
  Ok(())
}
