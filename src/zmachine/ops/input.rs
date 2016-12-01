use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::VM;

pub fn read_0x04<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  unimplemented!()
}
