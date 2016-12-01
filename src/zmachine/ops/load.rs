// YES

use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{BytePtr, VM, VariableRef};

pub fn storew_0x01<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  if operands[0..2].iter().any(|o| *o == Operand::Omitted) {
    panic!("3 operands required: {:?}", operands);
  }
  let array_val = operands[0].value(vm)?;
  let word_index_val = operands[1].value(vm)?;
  let word_ptr = BytePtr::new(array_val).inc_by(word_index_val * 2);
  let val = operands[2].value(vm)?;
  vm.write_memory(word_ptr, val)?;

  Ok(())
}

pub fn store_0x0d<T>(vm: &mut T, encoded_var_op: Operand, value_op: Operand) -> Result<()>
  where T: VM {
  let encoded_var_val = encoded_var_op.value(vm)?;
  let dst_var = VariableRef::decode(encoded_var_val as u8);
  let val = value_op.value(vm)?;
  vm.write_variable(dst_var, val)?;
  Ok(())
}

pub fn loadw_0x0f<T>(vm: &mut T,
                     array_op: Operand,
                     word_index_op: Operand,
                     result_ref: VariableRef)
                     -> Result<()>
  where T: VM {
  let array_val = array_op.value(vm)?;
  let word_index_val = word_index_op.value(vm)?;

  let result = vm.read_memory(BytePtr::new(array_val).inc_by(word_index_val * 2))?;

  vm.write_variable(result_ref, result)?;
  Ok(())
}

pub fn loadb_0x10<T>(vm: &mut T,
                     array_op: Operand,
                     byte_index_op: Operand,
                     result_ref: VariableRef)
                     -> Result<()>
  where T: VM {
  let array_val = array_op.value(vm)?;
  let byte_index_val = byte_index_op.value(vm)?;

  let result = vm.read_memory_u8(BytePtr::new(array_val).inc_by(byte_index_val))?;

  vm.write_variable(result_ref, result as u16)?;
  Ok(())
}

pub fn inc_0x05<T>(vm: &mut T, encoded_variable: Operand) -> Result<()>
  where T: VM {
  // TODO: test this.
  // TODO: make sure this is a signed computation. (-1++ = 0)
  let encoded = encoded_variable.value(vm)?;
  let variable = VariableRef::decode(encoded as u8);
  let old_value = vm.read_variable(variable)? as i16;
  vm.write_variable(variable, (old_value + 1) as u16)
}

#[cfg(test)]
mod test {
  use result::Result;
  use zmachine::ops::Operand;
  use zmachine::ops::testvm::TestVM;
  use zmachine::vm::{BytePtr, VM, VariableRef};

  #[test]
  fn test_storew() {
    let mut vm = TestVM::new();
    let val = 55;
    super::storew_0x01(&mut vm,
                       [Operand::SmallConstant(10),
                        Operand::LargeConstant(8),
                        Operand::SmallConstant(val),
                        Operand::Omitted]);
    assert_eq!(val as u16,
               vm.read_memory(BytePtr::new(10 + 2 * 8)).unwrap());
  }

  #[test]
  fn test_store_0x0d() {
    let mut vm = TestVM::new();
    let val = 55u16;
    let value_op = Operand::LargeConstant(val);
    let encoded_var_op = Operand::SmallConstant(VariableRef::encode(VariableRef::Stack));

    super::store_0x0d(&mut vm, encoded_var_op, value_op);
    assert_eq!(val, vm.pop_stack().unwrap());
  }

  #[test]
  fn test_loadw_0x0f() {
    let mut vm = TestVM::new();
    let val = 0x87u16;
    vm.write_memory(BytePtr::new(21 + 2 * 5), val).unwrap();
    let array_op = Operand::LargeConstant(21);
    let word_index_op = Operand::LargeConstant(5);
    super::loadw_0x0f(&mut vm, array_op, word_index_op, VariableRef::Stack);

    assert_eq!(val, vm.pop_stack().unwrap());
  }

  #[test]
  fn test_loadb_0x10() {
    let mut vm = TestVM::new();
    let val = 0x3456u16;
    vm.write_memory(BytePtr::new(60).inc_by(9), val);

    let array_op = Operand::SmallConstant(60);
    let byte_index_op = Operand::SmallConstant(9);
    super::loadb_0x10(&mut vm, array_op, byte_index_op, VariableRef::Stack);
    assert_eq!(0x34, vm.pop_stack().unwrap());
  }
}
