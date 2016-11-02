use result::{Error, Result};
use std::io::Read;

mod memory;
pub mod opcodes;
mod ops;
mod pc;
mod object_table;
mod stack;
mod vm;

use self::memory::Memory;
use self::object_table::ObjectTable;
use self::opcodes::{OpcodeRunner, Operand, VariableRef};
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
    match top_two_bits {
      0b11000000 => self.process_variable_opcode(first_byte),
      0b10000000 => self.process_short_opcode(first_byte),
      _ => self.process_long_opcode(first_byte),
    }
  }

  fn process_variable_opcode(&mut self, first_byte: u8) -> Result<()> {
    let opcode_number = first_byte & 0b00011111;
    // println!("var opcode number: {:x} @{:x}",
    //          opcode_number,
    //          self.pc.pc() - 1usize);
    let start_pc = self.pc.pc() - 1usize;
    if (first_byte & 0b00100000) == 0 {
      let (lhs, rhs) = self.read_2_operands();
      self.dispatch_2op(start_pc, opcode_number, lhs, rhs)
    } else {
      let operands = self.read_var_operands();
      match opcode_number {
        0x00 => ops::varops::call_0x00(self, operands),
        0x01 => ops::varops::storew_0x01(self, operands),
        0x03 => ops::varops::put_prop_0x03(self, operands),
        0x05 => ops::varops::print_char_0x05(self, operands),
        0x06 => ops::varops::print_num_0x06(self, operands),
        _ => Err(Error::UnknownOpcode("VAR", opcode_number, start_pc)),
      }
    }
  }

  fn read_2_operands(&mut self) -> (Operand, Operand) {
    let operand_types = self.next_pc_byte();
    let lhs = self.read_operand_of_type((operand_types & 0b11000000) >> 6);
    let rhs = self.read_operand_of_type((operand_types & 0b00110000) >> 4);

    // We ignore the next two operands, but they should be Omitted.
    // TODO: check that they are omitted.

    //    println!("VAR 2OP: {:?}/{:?}", lhs, rhs);
    (lhs, rhs)
  }

  fn read_var_operands(&mut self) -> [Operand; 4] {
    let operand_types = self.next_pc_byte();
    let operand1 = self.read_operand_of_type((operand_types & 0b11000000) >> 6);
    let operand2 = self.read_operand_of_type((operand_types & 0b00110000) >> 4);
    let operand3 = self.read_operand_of_type((operand_types & 0b00001100) >> 2);
    let operand4 = self.read_operand_of_type((operand_types & 0b00000011) >> 0);
    [operand1, operand2, operand3, operand4]
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
    // println!("short opcode number: {:x} @{:x}", op, self.pc.pc() - 1usize);
    let operand_type = (first_byte & 0b00110000) >> 4;
    let operand = self.read_operand_of_type(operand_type);
    match operand {
      Operand::Omitted => self.process_0op(op),
      _ => self.process_1op(op, operand),
    }
  }

  fn process_0op(&mut self, op: u8) -> Result<()> {
    try!(match op {
      0x00 => ops::zeroops::rtrue_0x00(self),
      0x02 => ops::zeroops::print_0x02(self),
      0x0b => ops::zeroops::new_line_0x0b(self),
      _ => {
        panic!("Unknown short 0op opcode: {:x} @{:x}", op, self.pc.pc() - 1);
      }
    });
    Ok(())
  }

  fn process_1op(&mut self, op: u8, operand: Operand) -> Result<()> {
    try!(match op {
      0x00 => ops::oneops::jz_0x00(self, operand),
      0x0b => ops::oneops::ret_0x0b(self, operand),
      0x0c => ops::oneops::jump_0x0c(self, operand),
      _ => {
        panic!("Unknown short 1op opcode: {:x} @{:x}", op, self.pc.pc() - 1);
      }
    });
    Ok(())
  }

  fn process_long_opcode(&mut self, first_byte: u8) -> Result<()> {
    let start_pc = self.pc.pc() - 1;
    let opcode_number = first_byte & 0b00011111;
    // println!("long opcode number: {:x} @{:x}",
    //          opcode_number,
    //          self.pc.pc() - 1usize);
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
    self.dispatch_2op(start_pc, opcode_number, first, second)
  }

  fn dispatch_2op(&mut self,
                  start_pc: usize,
                  opcode: u8,
                  lhs: Operand,
                  rhs: Operand)
                  -> Result<()> {
    match opcode {
      0x01 => ops::twoops::je_0x01(self, lhs, rhs),
      0x05 => ops::twoops::inc_chk_0x05(self, lhs, rhs),
      0x09 => ops::twoops::and_0x09(self, lhs, rhs),
      0x0a => ops::twoops::test_attr_0x0a(self, lhs, rhs),
      0x0d => ops::twoops::store_0x0d(self, lhs, rhs),
      0x0e => ops::twoops::insert_obj_0x0e(self, lhs, rhs),
      0x0f => ops::twoops::loadw_0x0f(self, lhs, rhs),
      0x10 => ops::twoops::loadb_0x10(self, lhs, rhs),
      0x14 => ops::twoops::add_0x14(self, lhs, rhs),
      0x15 => ops::twoops::sub_0x15(self, lhs, rhs),
      _ => panic!("Unknown long opcode: {:x} @{:x}", opcode, start_pc),
    }
  }
}

impl OpcodeRunner for ZMachine {
  fn read_memory(&self, byteaddress: usize) -> u16 {
    self.memory.u16_at_index(byteaddress)
  }

  fn write_memory(&mut self, byteaddress: usize, val: u16) {
    self.memory.set_u16_at_index(byteaddress, val);
  }

  fn read_memory_u8(&self, byteaddress: usize) -> u8 {
    self.memory.u8_at_index(byteaddress)
  }

  fn read_pc_byte(&mut self) -> u8 {
    self.next_pc_byte()
  }

  fn read_pc_word(&mut self) -> u16 {
    self.next_pc_word()
  }

  fn current_pc(&self) -> usize {
    self.pc.pc()
  }

  fn set_current_pc(&mut self, pc: usize) {
    self.pc.set_pc(pc);
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

  fn attributes(&mut self, object_number: u16) -> u32 {
    ObjectTable::new(&mut self.memory).attributes(object_number)
  }

  fn put_property(&mut self, object_index: u16, property_number: u16, value: u16) {
    ObjectTable::new(&mut self.memory).put_property(object_index, property_number, value);
  }

  fn insert_obj(&mut self, object_index: u16, dest_index: u16) {
    ObjectTable::new(&mut self.memory).insert_obj(object_index, dest_index);
  }

  fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> usize {
    let abbrev_table_offset = self.memory.abbrev_table_offset();
    let offset = abbrev_table_offset +
                 (32 * (abbrev_table as usize - 1) + abbrev_index as usize) * 2;
    let abbrev_addr = self.memory.u16_at_index(offset);
    // *2 because this is a word address.
    abbrev_addr as usize * 2
  }

  fn new_frame(&mut self, ret_pc: usize, num_locals: u8, result_location: u8) {
    self.stack.new_frame(ret_pc, num_locals, result_location);
  }

  fn pop_frame(&mut self, return_val: u16) -> (usize, VariableRef) {
    self.stack.pop_frame()
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
