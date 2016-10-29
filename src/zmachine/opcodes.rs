pub trait OpcodeRunner: Sized {
  fn read_pc_byte(&mut self) -> u8;
  fn read_pc_word(&mut self) -> u16;
  fn offset_pc(&mut self, offset: i16);

  fn pop_stack(&mut self) -> u16;
  fn push_stack(&mut self, val: u16);

  fn read_local(&self, local_idx: u8) -> u16;
  fn write_local(&mut self, local_idx: u8, val: u16);

  fn read_global(&self, global_idx: u8) -> u16;
  fn write_global(&mut self, global_idx: u8, val: u16);

  fn result_location(&mut self) -> VariableRef;

  fn write_result(&mut self, value: u16) {
    let result_location = self.result_location();
    self.write_to_variable(result_location, value);
  }

  fn read_variable(&mut self, variable: VariableRef) -> u16 {
    match variable {
      VariableRef::Stack => self.pop_stack(),
      VariableRef::Local(idx) => self.read_local(idx),
      VariableRef::Global(idx) => self.read_global(idx),
    }
  }

  fn write_to_variable(&mut self, variable: VariableRef, value: u16) {
    match variable {
      VariableRef::Stack => self.push_stack(value),
      VariableRef::Local(idx) => self.write_local(idx, value),
      VariableRef::Global(idx) => self.write_global(idx, value),
    }
  }
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum VariableRef {
  Stack,
  Local(u8),
  Global(u8),
}

impl VariableRef {
  pub fn decode(encoded: u8) -> VariableRef {
    match encoded {
      0x00 => VariableRef::Stack,
      0x01...0x0f => VariableRef::Local(encoded - 0x01),
      0x10...0xff => VariableRef::Global(encoded - 0x10),
      _ => panic!("What is this number: {}", encoded),
    }
  }

  pub fn encode(variable: VariableRef) -> u8 {
    match variable {
      VariableRef::Stack => 0,
      VariableRef::Local(local_idx) => 0x01 + local_idx,
      VariableRef::Global(global_idx) => 0x10 + global_idx,
    }
  }
}

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
  Two(Operand, Operand),
  //  Var(Operand, Operand, Operand, Operand),
  Var([Operand; 4]),
}

#[derive(Debug,Eq,PartialEq)]
pub enum Operand {
  LargeConstant(u16),
  SmallConstant(u8),
  Variable(VariableRef),
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
  pub fn value<T>(&self, runner: &mut T) -> u16
    where T: OpcodeRunner {
    match *self {
      Operand::LargeConstant(val) => val,
      Operand::SmallConstant(val) => val as u16,
      Operand::Variable(variable) => runner.read_variable(variable),
      Operand::Omitted => {
        panic!("Cannot read Omitted operand: {:?}", *self);
      }
    }
  }
}

#[cfg(test)]
pub mod test {
  use byteorder::{BigEndian, ByteOrder};
  use super::{OpcodeRunner, VariableRef};

  pub struct TestRunner {
    pub stack: Vec<u16>,
    pub locals: [u16; 15],
    pub globals: [u16; 240],
    pub pc: usize,
    pub pcbytes: Vec<u8>,
  }

  impl TestRunner {
    pub fn new() -> TestRunner {
      TestRunner {
        stack: Vec::new(),
        locals: [0; 15],
        globals: [0; 240],
        pc: 0,
        pcbytes: Vec::new(),
      }
    }

    pub fn current_pc(&self) -> usize {
      self.pc
    }

    pub fn set_result_location(&mut self, location: VariableRef) {
      self.set_pc_bytes(vec![VariableRef::encode(location)]);
    }

    pub fn set_pc_bytes(&mut self, bytes: Vec<u8>) {
      self.pcbytes = bytes;
      self.pc = 0;
    }

    pub fn set_jump_offset_byte(&mut self, offset: u8, polarity: bool) {
      let mut byte = 0b01000000u8;
      if polarity {
        byte |= 0b10000000;
      }
      byte |= offset & 0b00111111;
      self.set_pc_bytes(vec![byte]);
    }
  }

  impl OpcodeRunner for TestRunner {
    fn read_pc_byte(&mut self) -> u8 {
      let val = self.pcbytes[self.pc];
      self.pc += 1;
      val
    }

    fn read_pc_word(&mut self) -> u16 {
      let val = BigEndian::read_u16(&self.pcbytes[self.pc..]);
      self.pc += 2;
      val
    }

    fn offset_pc(&mut self, offset: i16) {
      self.pc = ((self.pc as i32) + (offset as i32)) as usize;
    }

    fn pop_stack(&mut self) -> u16 {
      self.stack.pop().unwrap()
    }

    fn push_stack(&mut self, val: u16) {
      self.stack.push(val);
    }

    fn read_local(&self, local_idx: u8) -> u16 {
      self.locals[local_idx as usize]
    }

    fn write_local(&mut self, local_idx: u8, val: u16) {
      self.locals[local_idx as usize] = val;
    }

    fn read_global(&self, global_idx: u8) -> u16 {
      self.globals[global_idx as usize]
    }

    fn write_global(&mut self, global_idx: u8, val: u16) {
      self.globals[global_idx as usize] = val;
    }

    fn result_location(&mut self) -> VariableRef {
      VariableRef::decode(self.read_pc_byte())
    }
  }

  #[test]
  fn test_variable_decode() {
    assert_eq!(VariableRef::Stack, VariableRef::decode(0x00));
    assert_eq!(VariableRef::Local(0), VariableRef::decode(0x01));
    assert_eq!(VariableRef::Local(5), VariableRef::decode(0x06));
    assert_eq!(VariableRef::Local(14), VariableRef::decode(0x0f));
    assert_eq!(VariableRef::Global(0), VariableRef::decode(0x10));
    assert_eq!(VariableRef::Global(80), VariableRef::decode(0x60));
    assert_eq!(VariableRef::Global(239), VariableRef::decode(0xff));
  }
}
