use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{VM, VariableRef, ZObject, ZObjectTable};

const BRANCH_POLARITY_MASK: u8 = 0b10000000;
const BRANCH_LENGTH_MASK: u8 = 0b01000000;

fn fourteen_bit_signed(b1: u8, b2: u8) -> i16 {
  // TODO: this is convoluted. Rewrite.
  let first = b1 & 0b00111111;
  let word: u16 = ((first as u16) << 8) + b2 as u16;

  // Converting to signed, check for the "high" bit (bit 14).
  if first & 0b00100000 == 0 {
    word as i16
  } else {
    // If negative, then sign extend and return.
    ((word | 0b1100000000000000) as i16)
  }
}

pub fn branch_on_condition<T>(vm: &mut T, condition: bool) -> Result<()>
  where T: VM {
  let first_label_byte = vm.read_pc_byte();
  let offset: i16;
  if first_label_byte & BRANCH_LENGTH_MASK == 0 {
    // two-byte, 14-bit signed offset
    let second_label_byte = vm.read_pc_byte();
    offset = fourteen_bit_signed(first_label_byte, second_label_byte);
  } else {
    // one-byte, 6-bit unsigned offset
    offset = (first_label_byte & 0b00111111) as i16;
  }

  // Branch on false iff BRANCH_POLARITY_MASK is 0.
  let branch_on = (first_label_byte & BRANCH_POLARITY_MASK) != 0;

  if condition == branch_on {
    if offset == 0 {
      // return false from the current routine
      vm.ret_value(0)?;
    } else if offset == 1 {
      // return true from the current routine
      vm.ret_value(1)?;
    } else {
      vm.offset_pc(offset - 2)?;
    }
  }
  Ok(())
}

pub fn branch_binop<F, T>(vm: &mut T, op1: Operand, op2: Operand, pred: F) -> Result<()>
  where F: Fn(i16, i16) -> bool,
        T: VM {
  // Rust will panic if we overflow, so do arithmetic as i32 and downcast.
  let lhs = op1.value(vm)? as i16;
  let rhs = op2.value(vm)? as i16;
  let condition = pred(lhs, rhs);

  branch_on_condition(vm, condition)
}

pub fn jz_0x00<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  let value = operand.value(vm)?;
  branch_on_condition(vm, value == 0)
}

pub fn je_0x01<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  let value = operands[0].value(vm)?;

  let mut condition = false;
  for operand in &operands[1..] {
    match *operand {
      Operand::Omitted => break,
      _ => {
        let operand_value = operand.value(vm)?;
        if value == operand_value {
          condition = true;
          break;
        }
      }
    }
  }

  branch_on_condition(vm, condition)
}

pub fn jl_0x02<T>(vm: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: VM {
  branch_binop(vm, lhs, rhs, |l, r| l < r)
}

pub fn jg_0x03<T>(vm: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: VM {
  branch_binop(vm, lhs, rhs, |l, r| l > r)
}

pub fn inc_chk_0x05<T>(vm: &mut T, var_op: Operand, value: Operand) -> Result<()>
  where T: VM {
  // TODO: test inc_chk_0x05
  let encoded = var_op.value(vm)?;
  let variable = VariableRef::decode(encoded as u8);
  let var_value = vm.read_variable(variable)?;
  let new_value = (var_value as u32 + 1) as u16;
  vm.write_variable(variable, new_value)?;
  branch_binop(vm, Operand::LargeConstant(new_value), value, |l, r| l > r)
}

pub fn dec_chk_0x04<T>(vm: &mut T, var_op: Operand, value: Operand) -> Result<()>
  where T: VM {
  // TODO: test dec_chk_0x04
  let encoded = var_op.value(vm)?;
  let variable = VariableRef::decode(encoded as u8);
  let var_value = vm.read_variable(variable)?;
  let new_value = (var_value as i16 as i32 - 1) as u16;
  vm.write_variable(variable, new_value)?;
  branch_binop(vm, Operand::LargeConstant(new_value), value, |l, r| l < r)
}

pub fn test_0x07<T>(vm: &mut T, test_op: Operand, mask_op: Operand) -> Result<()>
  where T: VM {
  branch_binop(vm, test_op, mask_op, |test, mask| test & mask == mask)
}

pub fn jump_0x0c<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  // TODO: test jump_0x0c
  let value = operand.value(vm)? as i16 as isize;
  let current_pc = vm.current_pc();
  let new_pc = ((current_pc as isize) + value) as usize - 2;
  vm.set_current_pc(new_pc)?;
  Ok(())
}

pub fn jin_0x06<T>(vm: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: VM {
  // TODO: test jin_0x06
  let child_number = lhs.value(vm)?;
  let parent_number = rhs.value(vm)?;
  let object_table = vm.object_table()?;
  let child_obj = object_table.object_with_number(child_number);
  let childs_parent_number = child_obj.parent(vm.object_storage());

  branch_on_condition(vm, parent_number == childs_parent_number)
}

pub fn get_child_0x02<T>(vm: &mut T, object_number: Operand, variable: VariableRef) -> Result<()>
  where T: VM {
  // TODO: test get_child_0x02
  let object_number = object_number.value(vm)?;
  let object_table = vm.object_table()?;

  let obj = object_table.object_with_number(object_number);
  let child_number = obj.child(vm.object_storage());

  vm.write_variable(variable, child_number)?;
  branch_on_condition(vm, child_number != 0)
}

pub fn get_sibling_0x01<T>(vm: &mut T,
                           object_number: Operand,
                           variable: VariableRef)
                           -> Result<()>
  where T: VM {
  // TODO: test get_sibling_0x01
  let object_number = object_number.value(vm)?;
  let object_table = vm.object_table()?;

  let obj = object_table.object_with_number(object_number);
  let sibling_number = obj.sibling(vm.object_storage());
  vm.write_variable(variable, sibling_number)?;
  branch_on_condition(vm, sibling_number != 0)
}

#[cfg(test)]
mod test {
  use super::*;
  use super::fourteen_bit_signed;
  use zmachine::ops::Operand;
  use zmachine::ops::testvm::TestVM;
  use zmachine::vm::{VM, VariableRef};

  #[test]
  fn test_14_bits() {
    assert_eq!(0, fourteen_bit_signed(0, 0));
    assert_eq!(1, fourteen_bit_signed(0, 1));
    assert_eq!(8191, fourteen_bit_signed(0b00011111, 0b11111111));
    assert_eq!(-1, fourteen_bit_signed(0b00111111, 0b11111111));
    assert_eq!(-8192, fourteen_bit_signed(0b00100000, 0b00000000));
  }

  #[test]
  fn test_branch_on_condition() {
    // Since almost all of the branch opcodes call branch_on_condition, we use
    // this test as a proxy for all of the weird cases with the 1 vs 2 byte
    // offset values and the condition bits. Then we make the bold assumption
    // that it is sufficient just to test a few cases for each opcode.
    let mut vm = TestVM::new();
    vm.set_jump_offset_byte(4, true);
    branch_on_condition(&mut vm, true);
    // offset - 2, plus 1 for the pc++ in branch_on_condition.
    // Other tests will be similar.
    assert_eq!(3, vm.current_pc());

    vm.set_jump_offset_byte(8, false);
    branch_on_condition(&mut vm, false);
    assert_eq!(7, vm.current_pc());

    vm.set_jump_offset_byte(12, true);
    branch_on_condition(&mut vm, false);
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(16, false);
    branch_on_condition(&mut vm, true);
    assert_eq!(1, vm.current_pc());

    // Test the two byte versions
    vm.set_jump_offset_word(7002, true);
    branch_on_condition(&mut vm, true);
    assert_eq!(7002, vm.current_pc());

    vm.set_jump_offset_word(8002, false);
    branch_on_condition(&mut vm, false);
    assert_eq!(8002, vm.current_pc());

    // Test the two-byte versions with negative numbers.
    // We have to make the pcbytes vec big enough to read with a two-byte pc.
    let mut vec = vec![0u8; 500];
    vm.set_jump_offset_word(-400, true);
    vec.append(&mut vm.pcbytes);
    vm.set_pcbytes(vec);
    vm.pc = 500;
    branch_on_condition(&mut vm, true);
    assert_eq!(100, vm.current_pc());
  }

  #[test]
  fn test_branch_binop() {
    let mut vm = TestVM::new();
    vm.set_jump_offset_byte(5, true);
    branch_binop(&mut vm,
                 Operand::SmallConstant(1),
                 Operand::LargeConstant(32),
                 |l, r| 1 == r - 31);
    assert_eq!(4, vm.current_pc());

    vm.set_jump_offset_byte(7, true);
    branch_binop(&mut vm,
                 Operand::SmallConstant(1),
                 Operand::LargeConstant(32),
                 |l, r| 1 == r - 310);
    assert_eq!(1, vm.current_pc());
  }

  #[test]
  fn test_jz_0x00() {
    let mut vm = TestVM::new();
    vm.set_jump_offset_byte(6, false);
    jz_0x00(&mut vm, Operand::SmallConstant(0x00)).unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(7, false);
    jz_0x00(&mut vm, Operand::SmallConstant(0x01)).unwrap();
    assert_eq!(6, vm.current_pc());

    vm.set_jump_offset_byte(8, true);
    jz_0x00(&mut vm, Operand::SmallConstant(0x00)).unwrap();
    assert_eq!(7, vm.current_pc());

    vm.set_jump_offset_byte(9, true);
    jz_0x00(&mut vm, Operand::SmallConstant(0x01)).unwrap();
    assert_eq!(1, vm.current_pc());
  }

  #[test]
  fn test_je_0x01() {
    let mut vm = TestVM::new();

    vm.set_jump_offset_byte(9, true);
    je_0x01(&mut vm,
            [Operand::SmallConstant(1),
             Operand::SmallConstant(1),
             Operand::Omitted,
             Operand::Omitted])
      .unwrap();
    assert_eq!(8, vm.current_pc());

    vm.set_jump_offset_byte(9, true);
    je_0x01(&mut vm,
            [Operand::SmallConstant(1),
             Operand::SmallConstant(2),
             Operand::Omitted,
             Operand::Omitted])
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(9, true);
    je_0x01(&mut vm,
            [Operand::SmallConstant(1),
             Operand::SmallConstant(32),
             Operand::SmallConstant(1),
             Operand::Omitted])
      .unwrap();
    assert_eq!(8, vm.current_pc());

    vm.set_jump_offset_byte(9, true);
    je_0x01(&mut vm,
            [Operand::SmallConstant(1),
             Operand::SmallConstant(64),
             Operand::SmallConstant(32),
             Operand::SmallConstant(1)])
      .unwrap();
    assert_eq!(8, vm.current_pc());

    vm.set_jump_offset_byte(9, true);
    je_0x01(&mut vm,
            [Operand::SmallConstant(1),
             Operand::SmallConstant(64),
             Operand::SmallConstant(32),
             Operand::SmallConstant(18)])
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(9, true);
    je_0x01(&mut vm,
            [Operand::SmallConstant(1),
             Operand::SmallConstant(64),
             Operand::Omitted,
             Operand::SmallConstant(1)])
      .unwrap();
    assert_eq!(1, vm.current_pc());
  }

  #[test]
  fn test_jl_0x02() {
    let mut vm = TestVM::new();

    vm.set_jump_offset_byte(5, true);
    jl_0x02(&mut vm,
            Operand::SmallConstant(1),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(4, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jl_0x02(&mut vm,
            Operand::SmallConstant(5),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jl_0x02(&mut vm,
            Operand::SmallConstant(3),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jl_0x02(&mut vm,
            Operand::LargeConstant(-3i16 as u16),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(4, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jl_0x02(&mut vm,
            Operand::LargeConstant(0),
            Operand::LargeConstant(-33i16 as u16))
      .unwrap();
    assert_eq!(1, vm.current_pc());
  }

  #[test]
  fn test_jg_0x03() {
    let mut vm = TestVM::new();

    vm.set_jump_offset_byte(5, true);
    jg_0x03(&mut vm,
            Operand::SmallConstant(1),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jg_0x03(&mut vm,
            Operand::SmallConstant(5),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(4, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jg_0x03(&mut vm,
            Operand::SmallConstant(3),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jg_0x03(&mut vm,
            Operand::LargeConstant(-3i16 as u16),
            Operand::SmallConstant(3))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(5, true);
    jg_0x03(&mut vm,
            Operand::LargeConstant(0),
            Operand::LargeConstant(-33i16 as u16))
      .unwrap();
    assert_eq!(4, vm.current_pc());
  }

}
