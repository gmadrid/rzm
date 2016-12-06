use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{VM, VariableRef};

pub fn push_0x08<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  let val = operands[0].value(vm)?;
  vm.push_stack(val)
}

pub fn pull_0x09<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  let encoded = operands[0].value(vm)?;
  let variable = VariableRef::decode(encoded as u8);
  let val = vm.pop_stack()?;
  vm.write_variable(variable, val)
}

pub fn pop_0x09<T>(vm: &mut T) -> Result<()>
  where T: VM {
  vm.pop_stack()?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use zmachine::ops::Operand;
  use zmachine::ops::testvm::TestVM;
  use zmachine::vm::{VM, VariableRef};

  #[test]
  fn test_push_0x08() {
    let mut vm = TestVM::new();
    let operands =
      [Operand::SmallConstant(8), Operand::Omitted, Operand::Omitted, Operand::Omitted];
    super::push_0x08(&mut vm, operands);

    assert_eq!(8, vm.pop_stack().unwrap());
  }

  #[test]
  fn test_pull_0x09() {
    let mut vm = TestVM::new();
    vm.push_stack(38);

    let global_idx = 3;
    let operands = [Operand::SmallConstant(VariableRef::encode(VariableRef::Global(global_idx))),
                    Operand::Omitted,
                    Operand::Omitted,
                    Operand::Omitted];
    super::pull_0x09(&mut vm, operands).unwrap();

    assert_eq!(38, vm.read_global(global_idx).unwrap());
  }
}
