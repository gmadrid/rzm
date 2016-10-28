use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operands};

pub fn add_0x14<T>(runner: &mut T, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  if let Operands::Two(op1, op2) = operands {
    // Rust will panic if we overflow, so do arithmetic as i32 and downcast.
    let lhs = op1.value(runner) as i16 as i32;
    let rhs = op2.value(runner) as i16 as i32;
    let result = (lhs + rhs) as u16;

    runner.write_result(result);
  } else {
    return Err(Error::BadOperands("add expects 2OP operands".to_string(), operands));
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use super::add_0x14;
  use zmachine::opcodes::{OpcodeRunner, Operand, Operands, VariableRef};
  use zmachine::opcodes::test::TestRunner;

  #[test]
  fn test_add_0x14() {
    let mut runner = TestRunner::new();
    add_0x14(&mut runner,
             Operands::Two(Operand::SmallConstant(32), Operand::SmallConstant(43)));
    assert_eq!(75u16, runner.pop_stack());
    add_0x14(&mut runner,
             Operands::Two(Operand::LargeConstant(-32i16 as u16),
                           Operand::SmallConstant(43)));
    assert_eq!(11u16, runner.pop_stack());
    add_0x14(&mut runner,
             Operands::Two(Operand::LargeConstant(-30000i16 as u16),
                           Operand::LargeConstant(-30000i16 as u16)));
    assert_eq!(-60000i32 as i16 as u16, runner.pop_stack());
    add_0x14(&mut runner,
             Operands::Two(Operand::LargeConstant(0xf000),
                           Operand::LargeConstant(0x3000)));
    assert_eq!(0x2000, runner.pop_stack());

    runner.write_local(2, 24);
    runner.write_global(8, 16);
    runner.set_result_location(VariableRef::Local(3));
    add_0x14(&mut runner,
             Operands::Two(Operand::Variable(VariableRef::Local(2)),
                           Operand::Variable(VariableRef::Global(8))));
    assert_eq!(40, runner.read_local(3));

    // test overwrite
    runner.write_local(5, 19);
    runner.set_result_location(VariableRef::Local(2));
    add_0x14(&mut runner,
             Operands::Two(Operand::Variable(VariableRef::Global(8)),
                           Operand::Variable(VariableRef::Local(5))));
    assert_eq!(35, runner.read_local(2));

    runner.write_global(150, 0xfffd);  // -3
    runner.write_global(165, 0x0005);
    runner.set_result_location(VariableRef::Global(180));
    add_0x14(&mut runner,
             Operands::Two(Operand::Variable(VariableRef::Global(150)),
                           Operand::Variable(VariableRef::Global(165))));
    assert_eq!(2, runner.read_global(180));
  }
}
