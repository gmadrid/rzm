use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};
use zmachine::ops::twoops::je_0x01;

pub fn jz_0x00<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  je_0x01(runner, operand, Operand::SmallConstant(0))
}
