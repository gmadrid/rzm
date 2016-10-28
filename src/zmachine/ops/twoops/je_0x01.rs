use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operands};

pub fn je_0x01<T>(runner: &mut T, opbyte: u8, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  if let Operands::Two(op1, op2) = operands {
    // Rust will panic if we overflow, so do arithmetic as i32 and downcast.
    let lhs = op1.value(runner) as i16 as i32;
    let rhs = op2.value(runner) as i16 as i32;
    let cmp = lhs == rhs;

    // Branch on false iff high bit is 0.
    let branch_on = (opbyte & 0x80) != 0;
    println!("je {:x} == {:x} when {}", lhs, rhs, branch_on);
    if cmp == branch_on {
      // do stuff here
      panic!("NOT READY YET");
    }
  } else {
    return Err(Error::BadOperands("add expects 2OP operands".to_string(), operands));
  }
  Ok(())
}
