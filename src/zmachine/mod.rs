use result::{Error, Result};
use std::io::Read;

mod memory;
pub mod opcodes;
mod ops;
mod pc;
mod stack;

use self::memory::Memory;
use self::opcodes::{OpcodeRunner, Operand, Operands, VariableRef};
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
    println!("var opcode number: {:x} @{:x}",
             opcode_number,
             self.pc.pc() - 1usize);
    let operands = if (first_byte & 0b00100000) == 0 {
      self.read_2_operands()
    } else {
      self.read_var_operands()
    };

    try!(match opcode_number {
      0 => self.call_var_224(operands),
      1 => ops::varops::storew_0x01(self, operands),
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
    Operands::Var([operand1, operand2, operand3, operand4])
  }

  fn read_operand_of_type(&mut self, operand_type: u8) -> Operand {
    let operand = match operand_type {
      0b00 => Operand::LargeConstant(self.next_pc_word()),
      0b01 => Operand::SmallConstant(self.next_pc_byte()),
      0b10 => Operand::Variable(VariableRef::decode(self.next_pc_byte())),
      0b11 => Operand::Omitted,
      _ => panic!("Unknown operand type: {:?}", operand_type),
    };
    operand
  }

  fn process_short_opcode(&mut self, first_byte: u8) -> Result<()> {
    let op = first_byte & 0b00001111;
    println!("short opcode number: {:x} @{:x}", op, self.pc.pc() - 1usize);
    let operand_type = (first_byte & 0b00110000) >> 4;
    let operand = self.read_operand_of_type(operand_type);
    match operand {
      Operand::Omitted => self.process_0op(op),
      _ => self.process_1op(op, operand),
    }
  }

  fn process_0op(&mut self, op: u8) -> Result<()> {
    unimplemented!();
  }

  fn process_1op(&mut self, op: u8, operand: Operand) -> Result<()> {
    try!(match op {
      0x00 => ops::oneops::jz_0x00(self, operand),
      _ => {
        panic!("Unknown short 1op opcode: {:x} @{:x}", op, self.pc.pc());
      }
    });
    Ok(())
  }

  fn process_long_opcode(&mut self, first_byte: u8) -> Result<()> {
    let opcode_number = first_byte & 0b00011111;
    println!("long opcode number: {:x} @{:x}",
             opcode_number,
             self.pc.pc() - 1usize);
    let first = self.read_operand_of_type(if first_byte & 0b01000000 == 0 {
      0b01
    } else {
      0b10
    });
    let second = self.read_operand_of_type(if first_byte & 0b00100000 == 0 {
      0b01
    } else {
      0b10
    });
    let operands = Operands::Two(first, second);
    try!(match opcode_number {
      0x01 => ops::twoops::je_0x01(self, operands),
      0x14 => ops::twoops::add_0x14(self, operands),
      0x15 => ops::twoops::sub_0x15(self, operands),
      _ => {
        panic!("Unknown long opcode: {:x}/{:x} @{:x}",
               opcode_number,
               first_byte,
               self.pc.pc())
      }
    });
    Ok(())
  }

  fn call_var_224(&mut self, operands: Operands) -> Result<()> {
    // TODO IMPORTANT, what about the result location. You need to read this
    // or the PC will be wrong.
    let result_location = self.next_pc_byte();

    // What about addresses in variables
    if let Operands::Var(operands) = operands {
      let ret_pc = self.pc.pc();

      let packed_addr = operands[0].value(self);
      self.pc.set_pc_to_packed_addr(packed_addr as usize);
      println!("packed addr{:#x}", packed_addr * 2);

      let num_args = self.next_pc_byte();
      println!("num args {:?}", num_args);

      // TODO: need to properly handle argument passing.
      self.stack.new_frame(ret_pc, num_args, result_location, &operands[1..]);

      for _ in 0..num_args {
        let arg = self.next_pc_word();
        println!("arg: {}", arg);
      }

      Ok(())
    } else {
      panic!("Non VAR operands received by call_var_224: {:?}", operands);
    }
  }
}

impl OpcodeRunner for ZMachine {
  fn write_memory(&mut self, byteaddress: usize, val: u16) {
    self.memory.set_u16_at_index(byteaddress, val);
  }

  fn read_pc_byte(&mut self) -> u8 {
    self.next_pc_byte()
  }

  fn read_pc_word(&mut self) -> u16 {
    self.next_pc_word()
  }

  fn offset_pc(&mut self, offset: i16) {
    let new_pc = (self.pc.pc() as i32 + offset as i32) as usize;
    self.pc.set_pc(new_pc);
  }

  fn pop_stack(&mut self) -> u16 {
    self.stack.pop_u16()
  }

  fn push_stack(&mut self, val: u16) {
    self.stack.push_u16(val);
  }

  fn read_local(&self, local_idx: u8) -> u16 {
    self.stack.read_local(local_idx)
  }

  fn write_local(&mut self, local_idx: u8, val: u16) {
    self.stack.write_local(local_idx, val);
  }

  fn read_global(&self, global_idx: u8) -> u16 {
    self.memory.read_global(global_idx)
  }

  fn write_global(&mut self, global_idx: u8, val: u16) {
    self.memory.write_global(global_idx, val);
  }

  fn result_location(&mut self) -> VariableRef {
    VariableRef::decode(self.next_pc_byte())
  }
}
