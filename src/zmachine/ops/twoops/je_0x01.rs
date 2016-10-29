use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operands};
use zmachine::ops::branch_binop;

const LABEL_POLARITY_MASK: u8 = 0b10000000;
const LABEL_LENGTH_MASK: u8 = 0b01000000;

pub fn je_0x01<T>(runner: &mut T, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  branch_binop(runner, operands, |l, r| l == r)
}

#[cfg(test)]
mod test {
  use super::je_0x01;
  use zmachine::opcodes::{OpcodeRunner, Operand, Operands, VariableRef};
  use zmachine::opcodes::test::TestRunner;

  #[test]
  fn test_je_false() {
    let mut runner = TestRunner::new();
    runner.set_jump_offset_byte(6, false);
    je_0x01(&mut runner,
            Operands::Two(Operand::SmallConstant(0x03), Operand::SmallConstant(0x03)));
    assert_eq!(1, runner.current_pc());

    runner.set_jump_offset_byte(6, false);
    je_0x01(&mut runner,
            Operands::Two(Operand::LargeConstant(0x03), Operand::SmallConstant(0x04)));
    assert_eq!(5, runner.current_pc());
  }

  #[test]
  fn test_je_true() {
    let mut runner = TestRunner::new();
    runner.set_jump_offset_byte(8, true);
    runner.write_local(3, 0x45);
    runner.push_stack(0x44);
    je_0x01(&mut runner,
            Operands::Two(Operand::Variable(VariableRef::Stack),
                          Operand::Variable(VariableRef::Local(3))));
    assert_eq!(1, runner.current_pc());

    runner.set_jump_offset_byte(8, true);
    runner.write_local(3, 0x45);
    runner.write_global(200, 0x45);
    je_0x01(&mut runner,
            Operands::Two(Operand::Variable(VariableRef::Global(200)),
                          Operand::Variable(VariableRef::Local(3))));
    assert_eq!(7, runner.current_pc());
  }

  #[test]
  fn test_je_two_bytes() {
    let mut runner = TestRunner::new();
    // TODO: write these tests
  }

  fn test_je() {
    let mut runner = TestRunner::new();

    // TODO: write these tests
    // test one one-byte
    // test two bytes
    // test two bytes negative

    // test ret false
    // test ret true
  }
}
