use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operand};

pub mod oneops;
pub mod twoops;
pub mod varops;

const BRANCH_POLARITY_MASK: u8 = 0b10000000;
const BRANCH_LENGTH_MASK: u8 = 0b01000000;

fn ret_value<T>(runner: &mut T, value: u16) -> Result<()>
  where T: OpcodeRunner {
  let (pc, result_var) = runner.pop_frame(value);
  runner.write_to_variable(result_var, value);
  runner.set_current_pc(pc);
  Ok(())
}

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

#[cfg(test)]
mod test {
  use super::fourteen_bit_signed;

  #[test]
  fn test_14_bits() {
    assert_eq!(0, fourteen_bit_signed(0, 0));
    assert_eq!(1, fourteen_bit_signed(0, 1));
    assert_eq!(8191, fourteen_bit_signed(0b00011111, 0b11111111));
    assert_eq!(-1, fourteen_bit_signed(0b00111111, 0b11111111));
    assert_eq!(-8192, fourteen_bit_signed(0b00100000, 0b00000000));
  }
}
