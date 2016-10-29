use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand, Operands};
use zmachine::ops::twoops::je_0x01;

pub fn jz_0x00<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  let operands = Operands::Two(operand, Operand::SmallConstant(0));
  je_0x01(runner, operands)
}
