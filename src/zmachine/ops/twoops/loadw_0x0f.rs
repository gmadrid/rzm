use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand, VariableRef};

pub fn loadw_0x0f<T>(runner: &mut T, array_op: Operand, word_index_op: Operand) -> Result<()>
  where T: OpcodeRunner {
  let array = array_op.value(runner);
  let word_index = word_index_op.value(runner);

  let result = runner.read_memory(array as usize + 2 * word_index as usize);
  runner.write_result(result);
  Ok(())
}

pub fn store_0x0d<T>(runner: &mut T, var_op: Operand, value: Operand) -> Result<()>
  where T: OpcodeRunner {
  let encoded = var_op.value(runner);
  let dst_var = VariableRef::decode(encoded as u8);
  let val = value.value(runner);
  runner.write_to_variable(dst_var, val);
  Ok(())
}
