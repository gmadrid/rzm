use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand, VariableRef};

pub fn storew_0x01<T>(runner: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: OpcodeRunner {
  if operands[0..2].iter().any(|o| *o == Operand::Omitted) {
    panic!("3 operands required: {:?}", operands);
  }
  let array = operands[0].value(runner) as usize;
  let word_index = operands[1].value(runner) as usize;
  let val = operands[2].value(runner);
  // println!("storew {:x}/{:x}/{:x} == {:?}",
  //          array,
  //          word_index,
  //          val,
  //          operands[2]);
  runner.write_memory(array + 2 * word_index, val);

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

pub fn loadw_0x0f<T>(runner: &mut T, array_op: Operand, word_index_op: Operand) -> Result<()>
  where T: OpcodeRunner {
  let array = array_op.value(runner);
  let word_index = word_index_op.value(runner);

  let result = runner.read_memory(array as usize + 2 * word_index as usize);
  runner.write_result(result);
  Ok(())
}

pub fn loadb_0x10<T>(runner: &mut T, array_op: Operand, byte_index_op: Operand) -> Result<()>
  where T: OpcodeRunner {
  let array = array_op.value(runner);
  let byte_index = byte_index_op.value(runner);

  let result = runner.read_memory_u8(array as usize + byte_index as usize);
  runner.write_result(result as u16);
  Ok(())
}
