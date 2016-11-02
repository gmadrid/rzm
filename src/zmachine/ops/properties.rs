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
use super::branch::branch_binop;
use zmachine::opcodes::{OpcodeRunner, Operand};

pub fn put_prop_0x03<T>(runner: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: OpcodeRunner {
  // TODO: Check all of these for Omitted.
  let object_index = operands[0].value(runner);
  let property_number = operands[1].value(runner);
  let new_value = operands[2].value(runner);

  runner.put_property(object_index, property_number, new_value);

  Ok(())
}

pub fn insert_obj_0x0e<T>(runner: &mut T, object_op: Operand, dest_op: Operand) -> Result<()>
  where T: OpcodeRunner {
  let object_index = object_op.value(runner);
  let dest_index = dest_op.value(runner);

  runner.insert_obj(object_index, dest_index);
  Ok(())
}

pub fn test_attr_0x0a<T>(runner: &mut T,
                         object_index: Operand,
                         attr_number: Operand)
                         -> Result<()>
  where T: OpcodeRunner {
  let object_index = object_index.value(runner);
  let attrs = runner.attributes(object_index);

  let attr_number = attr_number.value(runner);

  // attribute bits are 0..31 - the reverse of what I expect.
  let mask = 1u32 << (31u8 - attr_number as u8);
  let masked = attrs & mask;
  let val = masked != 0;

  branch_binop(runner,
               Operand::SmallConstant(val as u8),
               Operand::SmallConstant(0),
               |l, _| l != 0)
}
