use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operands};

const LABEL_POLARITY_MASK: u8 = 0b10000000;
const LABEL_LENGTH_MASK: u8 = 0b01000000;

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

pub fn je_0x01<T>(runner: &mut T, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  if let Operands::Two(op1, op2) = operands {
    // Rust will panic if we overflow, so do arithmetic as i32 and downcast.
    let lhs = op1.value(runner) as i16 as i32;
    let rhs = op2.value(runner) as i16 as i32;
    let cmp = lhs == rhs;

    let first_label_byte = runner.read_pc_byte();
    let mut offset: i16;
    if first_label_byte & LABEL_LENGTH_MASK == 0 {
      // two-byte, 14-bit signed offset
      let second_label_byte = runner.read_pc_byte();
      offset = fourteen_bit_signed(first_label_byte, second_label_byte);
    } else {
      // one-byte, 6-bit unsigned offset
      offset = first_label_byte as i16;
    }

    // Branch on false iff LABEL_POLARITY_MASK is 0.
    let branch_on = (first_label_byte & LABEL_POLARITY_MASK) != 0;
    println!("je {:x} == {:x} when {}", lhs, rhs, branch_on);

    if cmp == branch_on {
      if offset == 0 {
        // return false from the current routine
        unimplemented!();
      } else if offset == 1 {
        // return true from the current routine
        unimplemented!()
      } else {
        runner.offset_pc(offset);
      }
    }
  } else {
    return Err(Error::BadOperands("add expects 2OP operands".to_string(), operands));
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
