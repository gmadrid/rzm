use result::{Error, Result};
use std::io::Read;

mod memory;
mod opcodes;
mod pc;
mod stack;

use self::memory::Memory;
use self::opcodes::{Operand, Operands, Operation};
use self::pc::PC;
use self::stack::Stack;

const HEADER_SIZE: usize = 64;

pub struct ZMachine {
  memory: Memory,
  pc: PC,
  stack: Stack,
}

impl From<Vec<u8>> for ZMachine {
  // WARNING: From::from cannot fail, so this does no consistency checking.
  fn from(vec: Vec<u8>) -> ZMachine {
    let memory = Memory::from(vec);
    ZMachine::from(memory)
  }
}

impl From<Memory> for ZMachine {
  // WARNING: From::from cannot fail, so this does no consistency checking.
  fn from(memory: Memory) -> ZMachine {
    let pc = PC::new(memory.starting_pc());
    let mut zmachine = ZMachine {
      memory: memory,
      pc: pc,
      stack: Stack::new(0xfff0),
    };
    zmachine.reset_interpreter_flags();
    zmachine
  }
}

impl ZMachine {
  pub fn from_reader<T: Read>(mut reader: T) -> Result<ZMachine> {
    let mut zbytes = Vec::<u8>::new();
    let bytes_read = try!(reader.read_to_end(&mut zbytes));
    if bytes_read < HEADER_SIZE {
      return Err(Error::CouldNotReadHeader);
    }

    let memory = Memory::from(zbytes);
    let zmachine = ZMachine::from(memory);

    let expected_file_length = zmachine.memory.file_length();
    if expected_file_length != 0 && expected_file_length > bytes_read as u32 {
      // It's okay for the memory too be too big, but not too small.
      return Err(Error::ZFileTooShort);
    }

    Ok(zmachine)
  }

  fn reset_interpreter_flags(&mut self) {
    // The interpreter sets flags in the header to express its capabilities to the game.
    let flag1_mask = !0b01110000;  // no status line, no split screen, fixed-width font
    let old_val = self.memory.flag1();
    self.memory.set_flag1(old_val & flag1_mask);
  }

  pub fn run(&mut self) -> Result<()> {
    // TODO: check version number
    loop {
      try!(self.process_opcode());
    }
  }

  fn next_pc_byte(&mut self) -> u8 {
    self.pc.next_byte(&self.memory)
  }

  fn next_pc_word(&mut self) -> u16 {
    self.pc.next_word(&self.memory)
  }

  fn process_opcode(&mut self) -> Result<()> {
    let first_byte = self.next_pc_byte();
    let top_two_bits = first_byte & 0b11000000;

    // TODO: (v5) handle 0xbe - extended opcodes
    println!("\nNEW OPCODE");
    match top_two_bits {
      0b11000000 => self.process_variable_opcode(first_byte),
      0b10000000 => self.process_short_opcode(first_byte),
      _ => self.process_long_opcode(first_byte),
    }
  }

  fn process_variable_opcode(&mut self, first_byte: u8) -> Result<()> {
    let opcode_number = first_byte & 0b00011111;
    let operands = if (first_byte & 0b00100000) == 0 {
      self.read_2_operands()
    } else {
      self.read_var_operands()
    };
    let operation = Operation::new(first_byte, operands);
    println!("OPERATION: {:?}", operation);

    try!(match opcode_number {
      0 => self.call_var_224(operation),
      _ => Err(Error::UnknownOpcode(opcode_number, self.pc.pc())),
    });

    Ok(())
  }

  fn read_2_operands(&mut self) -> Operands {

    unimplemented!()
  }

  fn read_var_operands(&mut self) -> Operands {
    let operand_types = self.next_pc_byte();
    let operand1 = self.read_operand_of_type((operand_types & 0b11000000) >> 6);
    let operand2 = self.read_operand_of_type((operand_types & 0b00110000) >> 4);
    let operand3 = self.read_operand_of_type((operand_types & 0b00001100) >> 2);
    let operand4 = self.read_operand_of_type((operand_types & 0b00000011) >> 0);
    Operands::Var(operand1, operand2, operand3, operand4)
  }

  fn read_operand_of_type(&mut self, operand_type: u8) -> Operand {
    let operand = match operand_type {
      0b00 => Operand::LargeConstant(self.next_pc_word()),
      0b01 => Operand::SmallConstant(self.next_pc_byte()),
      0b10 => Operand::Variable,
      0b11 => Operand::Omitted,
      _ => panic!("Unknown operand type: {:?}", operand_type),
    };
    operand
  }

  fn process_short_opcode(&mut self, first_byte: u8) -> Result<()> {
    unimplemented!()
  }

  fn process_long_opcode(&mut self, first_byte: u8) -> Result<()> {
    unimplemented!()
  }

  fn call_var_224(&mut self, operation: Operation) -> Result<()> {
    // What about addresses in variables
    // TODO: convert Var to have a Vec not four operands
    if let Operands::Var(addr, a0, a1, a2) = operation.operands {
      let packed_addr = Operand::value(addr);
      self.pc.set_pc_to_packed_addr(packed_addr as usize);
      println!("packed addr{:#x}", packed_addr * 2);

      let num_args = self.next_pc_byte();
      println!("num args {:?}", num_args);

      for _ in 0..num_args {
        let arg = self.next_pc_word();
        println!("arg: {}", arg);
      }

      // TODO: make this non-recursive.
      //      try!(self.process_opcode());
      Ok(())
    } else {
      panic!("Non VAR operands received by call_var_224: {:?}",
             operation.operands);
    }
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn test_passes() {
    assert_eq!(1, 1);
  }
}
