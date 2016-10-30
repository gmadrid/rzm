use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};
use zmachine::ops::branch_binop;

pub fn je_0x01<T>(runner: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: OpcodeRunner {
  branch_binop(runner, lhs, rhs, |l, r| l == r)
}

#[cfg(test)]
mod test {
  use super::je_0x01;
  use zmachine::opcodes::{OpcodeRunner, Operand, VariableRef};
  use zmachine::opcodes::test::TestRunner;

  #[test]
  fn test_je_false() {
    let mut runner = TestRunner::new();
    runner.set_jump_offset_byte(6, false);
    je_0x01(&mut runner,
            Operand::SmallConstant(0x03),
            Operand::SmallConstant(0x03))
      .unwrap();
    assert_eq!(1, runner.current_pc());

    runner.set_jump_offset_byte(6, false);
    je_0x01(&mut runner,
            Operand::LargeConstant(0x03),
            Operand::SmallConstant(0x04))
      .unwrap();
    assert_eq!(5, runner.current_pc());
  }

  #[test]
  fn test_je_true() {
    let mut runner = TestRunner::new();
    runner.set_jump_offset_byte(8, true);
    runner.write_local(3, 0x45);
    runner.push_stack(0x44);
    je_0x01(&mut runner,
            Operand::Variable(VariableRef::Stack),
            Operand::Variable(VariableRef::Local(3)))
      .unwrap();
    assert_eq!(1, runner.current_pc());

    runner.set_jump_offset_byte(8, true);
    runner.write_local(3, 0x45);
    runner.write_global(200, 0x45);
    je_0x01(&mut runner,
            Operand::Variable(VariableRef::Global(200)),
            Operand::Variable(VariableRef::Local(3)))
      .unwrap();
    assert_eq!(7, runner.current_pc());
  }

  #[test]
  fn test_je_two_bytes() {
    let mut runner = TestRunner::new();
    // TODO: write these tests
    runner.set_jump_offset_word(400, true);
    je_0x01(&mut runner,
            Operand::SmallConstant(4),
            Operand::SmallConstant(4))
      .unwrap();
    assert_eq!(400, runner.current_pc());

    let mut vec = vec![0u8; 500];
    runner.set_jump_offset_word(-400, true);
    vec.append(&mut runner.pcbytes);
    runner.set_pc_bytes(vec);
    runner.pc = 500;
    je_0x01(&mut runner,
            Operand::SmallConstant(6),
            Operand::SmallConstant(6))
      .unwrap();
    assert_eq!(100, runner.current_pc());
  }

  #[test]
  fn test_je() {
    // TODO write these tests
    // test ret false
    // test ret true
  }
}
