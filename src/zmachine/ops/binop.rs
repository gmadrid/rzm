use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operand};

// The ZMachine works mostly with unsigned words. So, to perform a signed
// binary, we have to just through some hoops:
// Receive two unsigned words, convert to signed, perform the requested
// signed binary operation, then convert back to unsigned.
// TODO: comment this function better.
fn signed_binop<F, T>(runner: &mut T, lop: Operand, rop: Operand, binop: F) -> Result<()>
  where F: Fn(i32, i32) -> i32,
        T: OpcodeRunner {
  let lhs = lop.value(runner);
  let rhs = rop.value(runner);

  // First, treat the input bits as signed, then sign extend to 32 bits.
  // This is so that if we overflow, rust will not panic.
  let wide_lhs = lhs as i16 as i32;
  let wide_rhs = rhs as i16 as i32;
  let value = binop(wide_lhs, wide_rhs) as u16;

  runner.write_result(value);
  Ok(())
}

pub fn add_0x14<T>(runner: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: OpcodeRunner {
  signed_binop(runner, lhs, rhs, |l, r| l + r)
}

pub fn sub_0x15<T>(runner: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: OpcodeRunner {
  signed_binop(runner, lhs, rhs, |l, r| l - r)
}

#[cfg(test)]
mod test {
  use super::add_0x14;
  use zmachine::opcodes::{OpcodeRunner, Operand, VariableRef};
  use zmachine::opcodes::test::TestRunner;

  #[test]
  fn test_add_0x14() {
    let mut runner = TestRunner::new();

    runner.set_result_location(VariableRef::Stack);
    add_0x14(&mut runner,
             Operand::SmallConstant(32),
             Operand::SmallConstant(43))
      .unwrap();
    assert_eq!(75u16, runner.pop_stack());

    runner.set_result_location(VariableRef::Stack);
    add_0x14(&mut runner,
             Operand::LargeConstant(-32i16 as u16),
             Operand::SmallConstant(43))
      .unwrap();
    assert_eq!(11u16, runner.pop_stack());

    runner.set_result_location(VariableRef::Stack);
    add_0x14(&mut runner,
             Operand::LargeConstant(-30000i16 as u16),
             Operand::LargeConstant(-30000i16 as u16))
      .unwrap();
    assert_eq!(-60000i32 as i16 as u16, runner.pop_stack());

    runner.set_result_location(VariableRef::Stack);
    add_0x14(&mut runner,
             Operand::LargeConstant(0xf000),
             Operand::LargeConstant(0x3000))
      .unwrap();
    assert_eq!(0x2000, runner.pop_stack());

    runner.write_local(2, 24);
    runner.write_global(8, 16);
    runner.set_result_location(VariableRef::Local(3));
    add_0x14(&mut runner,
             Operand::Variable(VariableRef::Local(2)),
             Operand::Variable(VariableRef::Global(8)))
      .unwrap();
    assert_eq!(40, runner.read_local(3));

    // test overwrite
    runner.write_local(5, 19);
    runner.set_result_location(VariableRef::Local(2));
    add_0x14(&mut runner,
             Operand::Variable(VariableRef::Global(8)),
             Operand::Variable(VariableRef::Local(5)))
      .unwrap();
    assert_eq!(35, runner.read_local(2));

    runner.write_global(150, 0xfffd);  // -3
    runner.write_global(165, 0x0005);
    runner.set_result_location(VariableRef::Global(180));
    add_0x14(&mut runner,
             Operand::Variable(VariableRef::Global(150)),
             Operand::Variable(VariableRef::Global(165)))
      .unwrap();
    assert_eq!(2, runner.read_global(180));
  }
}
