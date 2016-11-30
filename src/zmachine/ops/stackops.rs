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

#[cfg(test)]
mod tests {
  // TODO: test push_0x08
  // TODO: test pull_0x09
}
