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
use zmachine::vm::{BytePtr, RawPtr, VM, VariableRef, ZObject, ZObjectTable, ZPropertyAccess,
                   ZPropertyTable};

pub fn put_prop_0x03<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  // TODO: Check all of these for Omitted.
  let object_index = operands[0].value(vm)?;
  let property_number = operands[1].value(vm)?;
  let new_value = operands[2].value(vm)?;

  let object_table = vm.object_table()?;
  let object = object_table.object_with_number(object_index);
  let property_table = object.property_table(vm.object_storage());
  Ok(property_table.set_property(property_number, new_value, vm.property_storage_mut()))
}

pub fn insert_obj_0x0e<T>(vm: &mut T, object_op: Operand, dest_op: Operand) -> Result<()>
  where T: VM {
  let object_index = object_op.value(vm)?;
  let dest_index = dest_op.value(vm)?;

  vm.object_table()?.insert_obj(object_index, dest_index, vm.object_storage_mut())
}

pub fn remove_obj_0x09<T>(vm: &mut T, object_op: Operand) -> Result<()>
  where T: VM {
  let object_number = object_op.value(vm)?;
  vm.object_table()?.remove_object_from_parent(object_number, vm.object_storage_mut())
}

// fn insert_obj(&mut self, object_number: u16, dest_number: u16) -> Result<()> {
//   let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
//   object_table.insert_obj(object_number, dest_number, &mut self.memory)
// }

pub fn test_attr_0x0a<T>(vm: &mut T, object_number: Operand, attr_number: Operand) -> Result<()>
  where T: VM {
  let object_number = object_number.value(vm)?;
  let object_table = vm.object_table()?;
  let obj = object_table.object_with_number(object_number);
  let attrs = obj.attributes(vm.object_storage());

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
  let object_table = vm.object_table()?;
  let obj = object_table.object_with_number(object_number);
  let attrs = {
    obj.attributes(vm.object_storage())
  };

  let attr_number = attr_number.value(vm)?;

  // attribute bits are 0..31 - the reverse of what I expect.
  let mask = 1u32 << (31u8 - attr_number as u8);
  let new_attrs = attrs | mask;
  obj.set_attributes(new_attrs, vm.object_storage_mut());
  Ok(())
}

pub fn clear_attr_0x0c<T>(vm: &mut T, object_number: Operand, attr_number: Operand) -> Result<()>
  where T: VM {
  let object_number = object_number.value(vm)?;
  let object_table = vm.object_table()?;
  let obj = object_table.object_with_number(object_number);
  let attrs = {
    obj.attributes(vm.object_storage())
  };

  let attr_number = attr_number.value(vm)?;

  // attribute bits are 0..31 - the reverse of what I expect.
  let mask = !(1u32 << (31u8 - attr_number as u8));
  let new_attrs = attrs & mask;
  obj.set_attributes(new_attrs, vm.object_storage_mut());
  Ok(())
}

pub fn get_parent_0x03<T>(vm: &mut T, object_number: Operand, variable: VariableRef) -> Result<()>
  where T: VM {
  // TODO: test get_parent_0x03
  let object_number = object_number.value(vm)?;
  let obj = vm.object_table()?.object_with_number(object_number);

  let parent_number = obj.parent(vm.object_storage());
  vm.write_variable(variable, parent_number)
}

pub fn get_prop_len_0x04<T>(vm: &mut T, prop_addr: Operand, variable: VariableRef) -> Result<()>
  where T: VM {
  let addr = prop_addr.value(vm)?;
  let val = if addr == 0 {
    0
  } else {
    // TODO: fix this abstraction violation. Storage of the length is impl. dependent.
    let ptr = BytePtr::new(addr - 1);
    let byte = vm.read_memory_u8(ptr)?;
    byte / 32 + 1
  };
  vm.write_variable(variable, val as u16)
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

  let property_value = {
    let object_table = vm.object_table()?;
    let object_storage = vm.object_storage();
    let property_storage = vm.property_storage();
    let property_table = object_table.object_with_number(object_number)
      .property_table(object_storage);
    let property = property_table.find_property(property_number, property_storage);

    match property {
      None => object_table.default_property_value(property_number, object_storage),
      Some((size, ptr)) => {
        match size {
          1 => property_storage.byte_property(ptr),
          2 => property_storage.word_property(ptr),
          _ => panic!("Bad size"),
        }
      }
    }
  };
  vm.write_variable(variable, property_value)
}

pub fn get_prop_addr_0x12<T>(vm: &mut T,
                             object_number: Operand,
                             property_number: Operand,
                             variable: VariableRef)
                             -> Result<()>
  where T: VM {
  // TODO: test get_prop_0x11
  let object_number = object_number.value(vm)?;
  let property_number = property_number.value(vm)?;

  let value = {
    let object_table = vm.object_table()?;
    let object_storage = vm.object_storage();
    let property_storage = vm.property_storage();
    let property_table = object_table.object_with_number(object_number)
      .property_table(object_storage);
    let property = property_table.find_property(property_number, property_storage);
    property.map(|(_, ptr)| RawPtr::from(ptr).into()).unwrap_or(0)
  };
  vm.write_variable(variable, value as u16)
}

pub fn get_next_prop_0x13<T>(vm: &mut T,
                             object_number: Operand,
                             property_number: Operand,
                             variable: VariableRef)
                             -> Result<()>
  where T: VM {
  let object_number = object_number.value(vm)?;
  let property_number = property_number.value(vm)?;

  let value = {
    let object_table = vm.object_table()?;
    let object_storage = vm.object_storage();
    let property_storage = vm.property_storage();
    let property_table = object_table.object_with_number(object_number)
      .property_table(object_storage);
    property_table.next_property(property_number, &property_storage)
  };
  info!(target: "pctrace", "get_next_prop: {}, {} => {}",
        object_number,
        property_number,
        value);
  vm.write_variable(variable, value)
}

#[cfg(test)]
mod tests {
  // TODO: test everything in this file.
}
