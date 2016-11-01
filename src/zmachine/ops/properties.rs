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
