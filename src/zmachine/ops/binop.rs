use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{VM, VariableRef};

// The ZMachine works mostly with unsigned words. So, to perform a signed
// binary, we have to jump through some hoops:
// Receive two unsigned words, convert to signed, perform the requested
// signed binary operation, then convert back to unsigned.
// TODO: comment this function better.
fn signed_binop<F, T>(vm: &mut T,
                      lop: Operand,
                      rop: Operand,
                      binop: F,
                      result_ref: VariableRef)
                      -> Result<()>
  where F: Fn(i32, i32) -> i32,
        T: VM {
  let lhs = lop.value(vm)?;
  let rhs = rop.value(vm)?;

  // First, treat the input bits as signed, then sign extend to 32 bits.
  // This is so that if we overflow, rust will not panic.
  let wide_lhs = lhs as i16 as i32;
  let wide_rhs = rhs as i16 as i32;
  let value = binop(wide_lhs, wide_rhs) as u16;

  vm.write_variable(result_ref, value)?;
  Ok(())
}

pub fn add_0x14<T>(vm: &mut T, lhs: Operand, rhs: Operand, result_ref: VariableRef) -> Result<()>
  where T: VM {
  signed_binop(vm, lhs, rhs, |l, r| l + r, result_ref)
}

pub fn sub_0x15<T>(vm: &mut T, lhs: Operand, rhs: Operand, result_ref: VariableRef) -> Result<()>
  where T: VM {
  signed_binop(vm, lhs, rhs, |l, r| l - r, result_ref)
}

pub fn and_0x09<T>(vm: &mut T, lhs: Operand, rhs: Operand, result_ref: VariableRef) -> Result<()>
  where T: VM {
  // TODO: convert this to use unsigned_binop
  signed_binop(vm, lhs, rhs, |l, r| l & r, result_ref)
}


#[cfg(test)]
mod test {
  use super::add_0x14;
  use zmachine::ops::Operand;
  use zmachine::ops::testvm::TestVM;
  use zmachine::vm::{VM, VariableRef};

  #[test]
  fn test_add_0x14() {
    let mut vm = TestVM::new();

    add_0x14(&mut vm,
             Operand::SmallConstant(32),
             Operand::SmallConstant(43),
             VariableRef::Stack)
      .unwrap();
    assert_eq!(75u16, vm.pop_stack().unwrap());

    add_0x14(&mut vm,
             Operand::LargeConstant(-32i16 as u16),
             Operand::SmallConstant(43),
             VariableRef::Stack)
      .unwrap();
    assert_eq!(11u16, vm.pop_stack().unwrap());

    add_0x14(&mut vm,
             Operand::LargeConstant(-30000i16 as u16),
             Operand::LargeConstant(-30000i16 as u16),
             VariableRef::Stack)
      .unwrap();
    assert_eq!(-60000i32 as i16 as u16, vm.pop_stack().unwrap());

    add_0x14(&mut vm,
             Operand::LargeConstant(0xf000),
             Operand::LargeConstant(0x3000),
             VariableRef::Stack)
      .unwrap();
    assert_eq!(0x2000, vm.pop_stack().unwrap());

    vm.write_local(2, 24);
    vm.write_global(8, 16);
    add_0x14(&mut vm,
             Operand::Variable(VariableRef::Local(2)),
             Operand::Variable(VariableRef::Global(8)),
             VariableRef::Local(3))
      .unwrap();
    assert_eq!(40, vm.read_local(3).unwrap());

    // test overwrite
    vm.write_local(5, 19);
    add_0x14(&mut vm,
             Operand::Variable(VariableRef::Global(8)),
             Operand::Variable(VariableRef::Local(5)),
             VariableRef::Local(2))
      .unwrap();
    assert_eq!(35, vm.read_local(2).unwrap());

    vm.write_global(150, 0xfffd);  // -3
    vm.write_global(165, 0x0005);
    add_0x14(&mut vm,
             Operand::Variable(VariableRef::Global(150)),
             Operand::Variable(VariableRef::Global(165)),
             VariableRef::Global(180))
      .unwrap();
    assert_eq!(2, vm.read_global(180).unwrap());
  }

  #[test]
  fn test_sub_0x15() {
    let mut vm = TestVM::new();
    super::sub_0x15(&mut vm,
                    Operand::LargeConstant(9),
                    Operand::LargeConstant(5),
                    VariableRef::Stack);
    assert_eq!(4, vm.pop_stack().unwrap());

    super::sub_0x15(&mut vm,
                    Operand::LargeConstant(-9i16 as u16),
                    Operand::LargeConstant(5),
                    VariableRef::Stack);
    assert_eq!(-14i16 as u16, vm.pop_stack().unwrap());

    super::sub_0x15(&mut vm,
                    Operand::LargeConstant(9),
                    Operand::LargeConstant(-5i16 as u16),
                    VariableRef::Stack);
    assert_eq!(14, vm.pop_stack().unwrap());

    super::sub_0x15(&mut vm,
                    Operand::LargeConstant(-9i16 as u16),
                    Operand::LargeConstant(-5i16 as u16),
                    VariableRef::Stack);
    assert_eq!(-4i16 as u16, vm.pop_stack().unwrap());

    // Test underflow
    super::sub_0x15(&mut vm,
                    Operand::LargeConstant(-32767i16 as u16),
                    Operand::LargeConstant(2 as u16),
                    VariableRef::Stack);
    assert_eq!(0x7fff, vm.pop_stack().unwrap());
  }

  #[test]
  fn test_and_0x09() {
    let mut vm = TestVM::new();
    super::and_0x09(&mut vm,
                    Operand::LargeConstant(0x0000u16),
                    Operand::LargeConstant(0xffffu16),
                    VariableRef::Stack);
    assert_eq!(0x0000u16, vm.pop_stack().unwrap());

    super::and_0x09(&mut vm,
                    Operand::LargeConstant(0x9191u16),
                    Operand::LargeConstant(0x8510u16),
                    VariableRef::Stack);
    assert_eq!(0x8110u16, vm.pop_stack().unwrap());
  }
}
