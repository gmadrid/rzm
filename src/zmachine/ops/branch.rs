use result::Result;
use super::ret_value;
use zmachine::opcodes::{OpcodeRunner, Operand};

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

fn branch_binop<F, T>(runner: &mut T, op1: Operand, op2: Operand, pred: F) -> Result<()>
  where F: Fn(i16, i16) -> bool,
        T: OpcodeRunner {
  // Rust will panic if we overflow, so do arithmetic as i32 and downcast.
  let lhs = op1.value(runner) as i16;
  let rhs = op2.value(runner) as i16;
  let cmp = pred(lhs, rhs);

  let first_label_byte = runner.read_pc_byte();
  let offset: i16;
  if first_label_byte & BRANCH_LENGTH_MASK == 0 {
    // two-byte, 14-bit signed offset
    let second_label_byte = runner.read_pc_byte();
    offset = fourteen_bit_signed(first_label_byte, second_label_byte);
  } else {
    // one-byte, 6-bit unsigned offset
    offset = (first_label_byte & 0b00111111) as i16;
  }

  // Branch on false iff LABEL_POLARITY_MASK is 0.
  let branch_on = (first_label_byte & BRANCH_POLARITY_MASK) != 0;
  println!("je {:x} == {:x} when {}", lhs, rhs, branch_on);

  if cmp == branch_on {
    if offset == 0 {
      // return false from the current routine
      try!(ret_value(runner, 0));
    } else if offset == 1 {
      // return true from the current routine
      try!(ret_value(runner, 1));
    } else {
      runner.offset_pc(offset - 2);
    }
  }
  Ok(())
}

pub fn jz_0x00<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  je_0x01(runner, operand, Operand::SmallConstant(0))
}

pub fn je_0x01<T>(runner: &mut T, lhs: Operand, rhs: Operand) -> Result<()>
  where T: OpcodeRunner {
  branch_binop(runner, lhs, rhs, |l, r| l == r)
}

pub fn jump_0x0c<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  let value = operand.value(runner) as i16 as isize;
  let current_pc = runner.current_pc();
  let new_pc = ((current_pc as isize) + value) as usize - 2;
  runner.set_current_pc(new_pc);
  Ok(())
}

#[cfg(test)]
mod test {
  use super::fourteen_bit_signed;
  use super::je_0x01;
  use zmachine::opcodes::{OpcodeRunner, Operand, VariableRef};
  use zmachine::opcodes::test::TestRunner;

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
