use result::{Error, Result};
use std::io::Read;

mod memory;
mod opcodes;
mod pc;

use self::memory::Memory;
use self::opcodes::{Operand, Operands, Operation};
use self::pc::PC;

const HEADER_SIZE: usize = 64;
// const STACK_SIZE: usize = 61440;

pub struct ZMachine {
  memory: Memory,
  pc: PC,
}

impl ZMachine {
  pub fn from_reader<T: Read>(mut reader: T) -> Result<ZMachine> {
    let mut zbytes = Vec::<u8>::new();
    let bytes_read = try!(reader.read_to_end(&mut zbytes));
    if bytes_read < HEADER_SIZE {
      return Err(Error::CouldNotReadHeader);
    }

    let memory: Memory = From::from(zbytes);
    let expected_file_length = memory.file_length();
    if expected_file_length != 0 && expected_file_length > bytes_read as u32 {
      // We can read extra bytes (and we often will).
      return Err(Error::ZFileTooShort);
    }
    let pc = PC::new(memory.starting_pc());

    let mut zmachine = ZMachine {
      memory: memory,
      pc: pc,
    };
    zmachine.reset_interpreter_flags();
    Ok(zmachine)
  }

  fn reset_interpreter_flags(&mut self) {
    // The interpreter sets flags in the header to express its capabilities to the game.
    let flag1_mask = !0b01110000;  // no status line, no split screen, fixed-width font
    let old_val = self.memory.flag1();
    self.memory.set_flag1(old_val & flag1_mask);
  }

  // pub fn machine_version(&self) -> u8 {
  //   self.header[0]
  // }

  pub fn run(&mut self) -> Result<()> {
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



    //    let first_byte = try!(self.pc.next_byte(&self.memory));


    // println!("first byte: {:x}/{:b}", first_byte, first_byte);

    // match first_byte {
    //   0x00...0x1f => {
    //     // long - 2OP
    //     //        try!(self.process_long_opcode(first_byte));
    //     let first_op = try!(self.pc.next_byte(&self.memory));
    //     let second_op = try!(self.pc.next_byte(&self.memory));
    //     let br_first = try!(self.pc.next_byte(&self.memory));
    //     let br_second = try!(self.pc.next_byte(&self.memory));
    //     println!("jg {} {} ({:#b}, {:x})",
    //              first_op,
    //              second_op,
    //              br_first,
    //              br_second);
    //   }
    //   0xe0...0xff => {
    //     // variable - VAR
    //     try!(self.process_variable_opcode(first_byte))
    //   }
    //   _ => {
    //     println!("Unknown opcode: {:?}", first_byte);
    //     return Err(Error::UnknownOpcode(first_byte, self.pc.pc()));

    //   }
    // }

    // Ok(())
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

    match opcode_number {
      0 => self.call_var_224(operation),
      _ => panic!("Operation unimplemented: {:?}", operation),
    };

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

  // fn process_variable_opcode(&mut self, first_byte: u8) -> Result<()> {
  //   let next_byte = try!(self.pc.next_byte(&self.memory));
  //   println!("{:x}/{:#b}", next_byte, next_byte);
  //   let address = try!{self.pc.next_word(&self.memory)};
  //   let arg1 = try!{self.pc.next_word(&self.memory)};
  //   let arg2 = try!{self.pc.next_word(&self.memory)};
  //   let store = try!{self.pc.next_byte(&self.memory)};
  //   println!("address: {:x}", address * 2);
  //   println!("arg1: {:x}", arg1);
  //   println!("arg2: {:x}", arg2);
  //   println!("store: {:x}", store);

  //   self.pc.set_pc((address * 2) as usize);
  //   let num_locals = try!(self.pc.next_byte(&self.memory));
  //   println!("\n\nnum_locals: {:?}", num_locals);
  //   for _ in 0..num_locals {
  //     let local = try!(self.pc.next_word(&self.memory));
  //     println!("local: {:x}/{:#b}", local, local);
  //   }
  //   //    let first_byte = try!(self.pc.next_byte(&self.memory));
  //   self.process_opcode();
  //   //    let next_byte = try!(self.pc.next_byte(&self.memory));
  //   //    let address = try!{self.pc.next_word(&self.memory)};
  //   //    println!("new_first_byte: {:x}/{:#b}", first_byte, first_byte);
  //   //    println!("new_next_byte: {:x}/{:#b}", next_byte, next_byte);
  //   //    println!("address: {:x}", address * 2);


  //   Ok(())
  // }

  // fn process_var_opcode(&mut self, first_byte: u8) -> Result<()> {
  //   let next_byte = self.memory[(self.pc - 64) as usize];
  //   self.pc += 1;
  //   println!("next_byte: {:b}", next_byte);
  //   let address = BigEndian::read_u16(&self.memory[(self.pc as usize)..]);
  //   println!("address: {:x}", address * 2);
  //   self.pc += 2;
  //   let arg1 = BigEndian::read_u16(&self.memory[(self.pc as usize)..]);
  //   self.pc += 2;
  //   let arg2 = BigEndian::read_u16(&self.memory[(self.pc as usize)..]);
  //   self.pc += 2;
  //   println!("arg1: {:x}", arg1);
  //   println!("arg2: {:x}", arg2);
  //   Ok(())
  // }
}
