use result::{Error, Result};
use std::io::Read;
use zmachine::ops;
use zmachine::ops::Operand;
use zmachine::vm::{RawPtr, VM, VariableRef, WordPtr};
use zmachine::vm::memory::Memory;
use zmachine::vm::mm_object_table::MemoryMappedObjectTable;
use zmachine::vm::object_table::{ZObject, ZObjectTable, ZPropertyTable};
use zmachine::vm::pc::PC;
use zmachine::vm::stack::Stack;

const HEADER_SIZE: usize = 64;

pub struct ZMachine {
  memory: Memory,
  pc: PC,
  stack: Stack,
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
    let bytes_read = reader.read_to_end(&mut zbytes)?;
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
      self.process_opcode()?
    }
  }

  fn process_opcode(&mut self) -> Result<()> {
    let pcvalue = usize::from(self.pc.pc());
    let first_byte = self.read_pc_byte();
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
    let start_pc: usize = self.pc.pc().into();
    let start_pc = start_pc - 1usize;
    //    println!("var opcode number: {:x} @{:x}", opcode_number, start_pc);
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
        0x08 => ops::varops::push_0x08(self, operands),
        0x09 => ops::varops::pull_0x09(self, operands),
        _ => Err(Error::UnknownOpcode("VAR", opcode_number, start_pc)),
      }
    }
  }

  fn read_var_operands(&mut self) -> [Operand; 4] {
    let operand_types = self.read_pc_byte();
    let operand1 = self.read_operand_of_type((operand_types & 0b11000000) >> 6);
    let operand2 = self.read_operand_of_type((operand_types & 0b00110000) >> 4);
    let operand3 = self.read_operand_of_type((operand_types & 0b00001100) >> 2);
    let operand4 = self.read_operand_of_type((operand_types & 0b00000011) >> 0);
    [operand1, operand2, operand3, operand4]
  }

  fn read_operand_of_type(&mut self, operand_type: u8) -> Operand {
    let operand = match operand_type {
      0b00 => Operand::LargeConstant(self.read_pc_word()),
      0b01 => Operand::SmallConstant(self.read_pc_byte()),
      0b10 => Operand::Variable(VariableRef::decode(self.read_pc_byte())),
      0b11 => Operand::Omitted,
      _ => panic!("Unknown operand type: {:?}", operand_type),
    };
    operand
  }

  fn read_2_operands(&mut self) -> (Operand, Operand) {
    // TODO: we should probably use read_var_operands for this,
    // then check that the last two are Omitted.
    let operand_types = self.read_pc_byte();
    let lhs = self.read_operand_of_type((operand_types & 0b11000000) >> 6);
    let rhs = self.read_operand_of_type((operand_types & 0b00110000) >> 4);

    // We ignore the next two operands, but they should be Omitted.
    // TODO: check that they are omitted.

    //    println!("VAR 2OP: {:?}/{:?}", lhs, rhs);
    (lhs, rhs)
  }

  fn process_short_opcode(&mut self, first_byte: u8) -> Result<()> {
    let op = first_byte & 0b00001111;
    // TODO: figure out how to write this better.
    let start_pc: usize = self.pc.pc().into();
    let start_pc = start_pc - 1usize;
    //    println!("short opcode number: {:x} @{:x}", op, start_pc);
    let operand_type = (first_byte & 0b00110000) >> 4;
    let operand = self.read_operand_of_type(operand_type);
    match operand {
      Operand::Omitted => self.process_0op(start_pc, op),
      _ => self.process_1op(start_pc, op, operand),
    }
  }

  fn process_0op(&mut self, start_pc: usize, op: u8) -> Result<()> {

    match op {
      0x00 => ops::zeroops::rtrue_0x00(self),
      0x02 => ops::zeroops::print_0x02(self),
      0x0b => ops::zeroops::new_line_0x0b(self),
      _ => {
        panic!("Unknown short 0op opcode: {:x} @{:x}", op, start_pc);
      }
    }
  }

  fn process_1op_with_return(&mut self,
                             operand: Operand,
                             op_func: &Fn(&mut Self, Operand, VariableRef) -> Result<()>)
                             -> Result<()> {
    let encoded = self.read_pc_byte();
    let variable = VariableRef::decode(encoded);
    op_func(self, operand, variable)
  }

  fn process_1op(&mut self, start_pc: usize, op: u8, operand: Operand) -> Result<()> {
    match op {
      0x00 => ops::oneops::jz_0x00(self, operand),
      0x03 => self.process_1op_with_return(operand, &ops::oneops::get_parent_0x03),
      0x0a => ops::oneops::print_obj_0x0a(self, operand),
      0x0b => ops::oneops::ret_0x0b(self, operand),
      0x0c => ops::oneops::jump_0x0c(self, operand),
      _ => {
        panic!("Unknown short 1op opcode: {:x} @{:x}", op, start_pc);
      }
    }
  }

  fn process_long_opcode(&mut self, first_byte: u8) -> Result<()> {
    let start_pc: usize = self.pc.pc().into();
    let start_pc = start_pc - 1;
    let opcode_number = first_byte & 0b00011111;
    //    println!("long opcode number: {:x} @{:x}", opcode_number, start_pc);
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

  fn dispatch_2op_with_return(&mut self,
                              lhs: Operand,
                              rhs: Operand,
                              op_func: &Fn(&mut Self, Operand, Operand, VariableRef) -> Result<()>)
                              -> Result<()> {
    let encoded = self.read_pc_byte();
    let variable = VariableRef::decode(encoded);
    op_func(self, lhs, rhs, variable)
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
      0x06 => ops::twoops::jin_0x06(self, lhs, rhs),
      0x09 => self.dispatch_2op_with_return(lhs, rhs, &ops::twoops::and_0x09),
      0x0a => ops::twoops::test_attr_0x0a(self, lhs, rhs),
      0x0b => ops::twoops::set_attr_0x0b(self, lhs, rhs),
      0x0d => ops::twoops::store_0x0d(self, lhs, rhs),
      0x0e => ops::twoops::insert_obj_0x0e(self, lhs, rhs),
      0x0f => self.dispatch_2op_with_return(lhs, rhs, &ops::twoops::loadw_0x0f),
      0x10 => self.dispatch_2op_with_return(lhs, rhs, &ops::twoops::loadb_0x10),
      0x14 => self.dispatch_2op_with_return(lhs, rhs, &ops::twoops::add_0x14),
      0x15 => self.dispatch_2op_with_return(lhs, rhs, &ops::twoops::sub_0x15),
      _ => panic!("Unknown long opcode: {:x} @{:x}", opcode, start_pc),
    }
  }
}

impl VM for ZMachine {
  fn read_pc_byte(&mut self) -> u8 {
    self.pc.next_byte(&self.memory)
  }

  fn read_pc_word(&mut self) -> u16 {
    self.pc.next_word(&self.memory)
  }

  fn current_pc(&self) -> usize {
    self.pc.pc().into()
  }

  // TODO: make this take a RawPtr.
  fn set_current_pc(&mut self, pc: usize) -> Result<()> {
    self.pc.set_pc(RawPtr::new(pc));
    Ok(())
  }

  fn offset_pc(&mut self, offset: i16) -> Result<()> {
    let pc: usize = self.pc.pc().into();
    let new_pc = (pc as i32 + offset as i32) as usize;
    self.pc.set_pc(RawPtr::new(new_pc));
    Ok(())
  }

  fn new_frame(&mut self,
               ret_pc: usize,
               num_locals: u8,
               result_location: VariableRef)
               -> Result<()> {
    self.stack.new_frame(ret_pc, num_locals, result_location);
    Ok(())
  }

  fn pop_frame(&mut self) -> Result<(usize, VariableRef)> {
    Ok(self.stack.pop_frame())
  }

  fn pop_stack(&mut self) -> Result<u16> {
    Ok(self.stack.pop_u16())
  }

  fn push_stack(&mut self, val: u16) -> Result<()> {
    self.stack.push_u16(val);
    Ok(())
  }

  fn read_local(&self, local_idx: u8) -> Result<u16> {
    Ok(self.stack.read_local(local_idx))
  }

  fn write_local(&mut self, local_idx: u8, val: u16) -> Result<()> {
    self.stack.write_local(local_idx, val);
    Ok(())
  }

  fn read_global(&self, global_idx: u8) -> Result<u16> {
    Ok(self.memory.read_global(global_idx))
  }

  fn write_global(&mut self, global_idx: u8, val: u16) -> Result<()> {
    self.memory.write_global(global_idx, val);
    Ok(())
  }

  fn read_memory<T>(&self, ptr: T) -> Result<u16>
    where T: Into<RawPtr> {
    Ok(self.memory.u16_at(ptr))
  }

  fn write_memory<T>(&mut self, ptr: T, val: u16) -> Result<()>
    where T: Into<RawPtr> {
    self.memory.set_u16_at(val, ptr);
    Ok(())
  }

  fn read_memory_u8<T>(&self, ptr: T) -> Result<u8>
    where T: Into<RawPtr> {
    Ok(self.memory.u8_at(ptr))
  }

  fn parent_number(&self, object_number: u16) -> Result<u16> {
    let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
    let object = object_table.object_with_number(object_number);
    Ok(object.parent(&self.memory))
  }

  fn attributes(&mut self, object_number: u16) -> Result<u32> {
    let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
    let object = object_table.object_with_number(object_number);
    Ok(object.attributes(&self.memory))
  }

  fn set_attributes(&mut self, object_number: u16, attrs: u32) -> Result<()> {
    let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
    let object = object_table.object_with_number(object_number);
    object.set_attributes(attrs, &mut self.memory);
    Ok(())
  }

  fn object_name(&self, object_number: u16) -> Result<RawPtr> {
    let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
    let object = object_table.object_with_number(object_number);
    let property_table = object.property_table(&self.memory);
    Ok(property_table.name_ptr(&self.memory).into())
  }

  fn put_property(&mut self, object_number: u16, property_index: u16, value: u16) -> Result<()> {
    let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
    let object = object_table.object_with_number(object_number);
    let property_table = object.property_table(&self.memory);
    Ok(property_table.set_property(property_index, value, &mut self.memory))
  }

  fn insert_obj(&mut self, object_number: u16, dest_number: u16) -> Result<()> {
    let object_table = MemoryMappedObjectTable::new(self.memory.property_table_ptr());
    object_table.insert_obj(object_number, dest_number, &mut self.memory)
  }

  fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> Result<WordPtr> {
    let abbrev_table_ptr = self.memory.abbrev_table_ptr();
    let abbrev_entry_ptr =
      abbrev_table_ptr.inc_by((32 * (abbrev_table as u16 - 1) + abbrev_index as u16) * 2);
    let abbrev_addr = self.memory.u16_at(abbrev_entry_ptr);
    Ok(WordPtr::new(abbrev_addr))
  }
}
