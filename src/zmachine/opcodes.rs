pub trait OpcodeRunner: Sized {
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
  use super::{OpcodeRunner, VariableRef};

  pub struct TestRunner {
    pub stack: Vec<u16>,
    pub locals: [u16; 15],
    pub globals: [u16; 240],
    pub result_location: Option<VariableRef>,
  }

  impl TestRunner {
    pub fn new() -> TestRunner {
      TestRunner {
        stack: Vec::new(),
        locals: [0; 15],
        globals: [0; 240],
        result_location: Some(VariableRef::Stack),
      }
    }

    pub fn set_result_location(&mut self, location: VariableRef) {
      self.result_location = Some(location);
    }

    pub fn clear_result_location(&mut self) {
      self.result_location = None;
    }
  }

  impl OpcodeRunner for TestRunner {
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
      // should panic if called when not expected
      self.result_location.unwrap()
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
