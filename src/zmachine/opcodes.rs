#[derive(Debug)]
pub struct Operation {
  pub opcode: u8,
  pub operands: Operands,
}

#[derive(Debug)]
enum OpForm {
  Long,
  Short,
  Variable,
  Extended,
}

#[derive(Debug)]
pub enum Operands {
  None,
  One,
  Two,
  Var(Operand, Operand, Operand, Operand),
}

#[derive(Debug)]
pub enum Operand {
  LargeConstant(u16),
  SmallConstant(u8),
  Variable,
  Omitted,
}

impl Operation {
  pub fn new(opcode: u8, operands: Operands) -> Operation {
    Operation {
      opcode: opcode,
      operands: operands,
    }
  }
}

impl Operand {
  pub fn value(operand: Operand) -> u16 {
    match operand {
      Operand::LargeConstant(val) => val,
      Operand::SmallConstant(val) => val as u16,
      _ => {
        panic!("CANNOT GET VALUE FOR OPERAND: {:?}", operand);
      }
    }
  }
}
