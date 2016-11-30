// put_prop
//
// VAR:227 3 put_prop object property value
//
// Writes the given value to the given property of the given object.
//
// If the property does not exist for that object, the interpreter should halt
// with a suitable error message.
//
// If the property length is 1, then the interpreter should store only the least
// significant byte of the value.
//
// (For instance, storing -1 into a 1-byte property results in the property value 255.)
// As with get_prop the property length must not be more than 2: if it is, the behaviour
// of the opcode is undefined.
//

use result::Result;
use zmachine::ops::Operand;
use zmachine::ops::branch::branch_binop;
use zmachine::vm::{VM, VariableRef};

pub fn put_prop_0x03<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  // TODO: Check all of these for Omitted.
  let object_index = operands[0].value(vm)?;
  let property_number = operands[1].value(vm)?;
  let new_value = operands[2].value(vm)?;

  vm.put_property(object_index, property_number, new_value)?;

  Ok(())
}

pub fn insert_obj_0x0e<T>(vm: &mut T, object_op: Operand, dest_op: Operand) -> Result<()>
  where T: VM {
  let object_index = object_op.value(vm)?;
  let dest_index = dest_op.value(vm)?;

  vm.insert_obj(object_index, dest_index)?;
  Ok(())
}

pub fn test_attr_0x0a<T>(vm: &mut T, object_index: Operand, attr_number: Operand) -> Result<()>
  where T: VM {
  let object_index = object_index.value(vm)?;
  let attrs = vm.attributes(object_index)?;

  let attr_number = attr_number.value(vm)?;

  // attribute bits are 0..31 - the reverse of what I expect.
  let mask = 1u32 << (31u8 - attr_number as u8);
  let masked = attrs & mask;
  let val = masked != 0;

  branch_binop(vm,
               Operand::SmallConstant(val as u8),
               Operand::SmallConstant(0),
               |l, _| l != 0)
}

pub fn set_attr_0x0b<T>(vm: &mut T, object_number: Operand, attr_number: Operand) -> Result<()>
  where T: VM {
  let object_number = object_number.value(vm)?;
  let attr_number = attr_number.value(vm)?;

  let attrs = vm.attributes(object_number)?;

  // attribute bits are 0..31 - the reverse of what I expect.
  let mask = 1u32 << (31u8 - attr_number as u8);
  let new_attrs = attrs | mask;
  vm.set_attributes(object_number, new_attrs)
}

pub fn get_parent_0x03<T>(vm: &mut T, object_number: Operand, variable: VariableRef) -> Result<()>
  where T: VM {
  // TODO: test get_parent_0x03
  let object_number = object_number.value(vm)?;
  let parent_number = vm.parent_number(object_number)?;
  vm.write_variable(variable, parent_number)
}

pub fn get_prop_0x11<T>(vm: &mut T,
                        object_number: Operand,
                        property_number: Operand,
                        variable: VariableRef)
                        -> Result<()>
  where T: VM {
  // TODO: test get_prop_0x11
  let object_number = object_number.value(vm)?;
  let property_number = property_number.value(vm)?;
  let value = vm.get_property(object_number, property_number)?;
  vm.write_variable(variable, value)
}


#[cfg(test)]
mod tests {
  // TODO: test everything in this file.
}
