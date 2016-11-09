use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{VM, VariableRef};

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

pub fn branch_binop<F, T>(vm: &mut T, op1: Operand, op2: Operand, pred: F) -> Result<()>
  where F: Fn(i16, i16) -> bool,
        T: VM {
  // Rust will panic if we overflow, so do arithmetic as i32 and downcast.
  let lhs = try!(op1.value(vm)) as i16;
  let rhs = try!(op2.value(vm)) as i16;
  let cmp = pred(lhs, rhs);

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

  // Branch on false iff LABEL_POLARITY_MASK is 0.
  let branch_on = (first_label_byte & BRANCH_POLARITY_MASK) != 0;
  //  println!("je {:x} == {:x} when {}", lhs, rhs, branch_on);

  if cmp == branch_on {
    if offset == 0 {
      // return false from the current routine
      try!(vm.ret_value(0));
    } else if offset == 1 {
      // return true from the current routine
      try!(vm.ret_value(1));
    } else {
      vm.offset_pc(offset - 2);
    }
  }
  Ok(())
}

pub fn jz_0x00<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  je_0x01(vm, operand, Operand::SmallConstant(0))
}

pub fn je_0x01<T>(vm: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: VM {
  branch_binop(vm, lhs, rhs, |l, r| l == r)
}

pub fn inc_chk_0x05<T>(vm: &mut T, var_op: Operand, value: Operand) -> Result<()>
  where T: VM {
  let encoded = try!(var_op.value(vm));
  let variable = VariableRef::decode(encoded as u8);
  let var_value = try!(vm.read_variable(variable));
  let cmp_value = try!(value.value(vm));
  try!(vm.write_variable(variable, var_value + 1));
  branch_binop(vm,
               Operand::LargeConstant(var_value + 1),
               value,
               |l, r| l > r)
}

pub fn jump_0x0c<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  let value = try!(operand.value(vm)) as i16 as isize;
  let current_pc = vm.current_pc();
  let new_pc = ((current_pc as isize) + value) as usize - 2;
  try!(vm.set_current_pc(new_pc));
  Ok(())
}

#[cfg(test)]
mod test {
  use super::fourteen_bit_signed;
  use super::je_0x01;
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
  fn test_je_false() {
    let mut vm = TestVM::new();
    vm.set_jump_offset_byte(6, false);
    je_0x01(&mut vm,
            Operand::SmallConstant(0x03),
            Operand::SmallConstant(0x03))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(6, false);
    je_0x01(&mut vm,
            Operand::LargeConstant(0x03),
            Operand::SmallConstant(0x04))
      .unwrap();
    assert_eq!(5, vm.current_pc());
  }

  #[test]
  fn test_je_true() {
    let mut vm = TestVM::new();
    vm.set_jump_offset_byte(8, true);
    vm.write_local(3, 0x45);
    vm.push_stack(0x44);
    je_0x01(&mut vm,
            Operand::Variable(VariableRef::Stack),
            Operand::Variable(VariableRef::Local(3)))
      .unwrap();
    assert_eq!(1, vm.current_pc());

    vm.set_jump_offset_byte(8, true);
    vm.write_local(3, 0x45);
    vm.write_global(200, 0x45);
    je_0x01(&mut vm,
            Operand::Variable(VariableRef::Global(200)),
            Operand::Variable(VariableRef::Local(3)))
      .unwrap();
    assert_eq!(7, vm.current_pc());
  }

  #[test]
  fn test_je_two_bytes() {
    // TODO: check that these are testing correctly.
    let mut vm = TestVM::new();
    // TODO: write these tests
    vm.set_jump_offset_word(400, true);
    je_0x01(&mut vm,
            Operand::SmallConstant(4),
            Operand::SmallConstant(4))
      .unwrap();
    assert_eq!(400, vm.current_pc());

    let mut vec = vec![0u8; 500];
    vm.set_jump_offset_word(-400, true);
    vec.append(&mut vm.pcbytes);
    vm.set_pcbytes(vec);
    vm.pc = 500;
    je_0x01(&mut vm,
            Operand::SmallConstant(6),
            Operand::SmallConstant(6))
      .unwrap();
    assert_eq!(100, vm.current_pc());
  }

  #[test]
  fn test_je() {
    // TODO write these tests
    // test ret false
    // test ret true
  }
}
