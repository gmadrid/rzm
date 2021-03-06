use ncurses::{A_REVERSE, WINDOW, endwin, getmaxyx, getyx, initscr, mvwprintw, newwin, noecho, raw,
              refresh, scrollok, stdscr, waddch, wattron, wmove, wprintw, wrefresh};
use rand::{Rng, SeedableRng, StdRng};
use result::{Error, Result};
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Write};
use std::rc::Rc;
use zmachine::ops;
use zmachine::ops::Operand;
use zmachine::vm::{BytePtr, RawPtr, VM, VariableRef, WordPtr};
use zmachine::vm::dictionary::Dictionary;
use zmachine::vm::memory::Memory;
use zmachine::vm::mm_object_table::{MemoryMappedObjectTable, MemoryMappedPropertyTable};
use zmachine::vm::pc::PC;
use zmachine::vm::quetzal::Quetzal;
use zmachine::vm::stack::Stack;
use zmachine::vm::zreadline::ZReadline;
use zmachine::zconfig::{ZConfig, ZDefaults};

const HEADER_SIZE: usize = 64;

pub struct ZMachine {
  memory: Rc<RefCell<Memory>>,
  pc: PC,
  stack: Stack,
  dict: Dictionary,

  status_window: Option<WINDOW>,
  main_window: Option<WINDOW>,
  num_rows: i32,
  num_cols: i32,

  rng: StdRng,
}

impl ZMachine {
  pub fn from_memory<T>(memory: Memory, config: &T) -> ZMachine
    where T: ZConfig {
    let memory_rc = Rc::new(RefCell::new(memory));
    let pc = PC::new(memory_rc.borrow().starting_pc(), memory_rc.clone());
    let rng = StdRng::new().unwrap();
    let mut zmachine = ZMachine {
      memory: memory_rc.clone(),
      pc: pc,
      stack: Stack::new(config.stack_size().unwrap()),
      dict: Dictionary::new(&memory_rc.borrow()),
      status_window: None,
      main_window: None,
      num_rows: 0,
      num_cols: 0,
      rng: rng,
    };
    zmachine.reset_interpreter_flags();
    zmachine
  }

  pub fn from_reader<T: Read>(mut reader: T) -> Result<ZMachine> {
    let mut zbytes = Vec::<u8>::new();
    let bytes_read = reader.read_to_end(&mut zbytes)?;
    if bytes_read < HEADER_SIZE {
      return Err(Error::CouldNotReadHeader);
    }

    let config = ZDefaults::new();
    let memory = Memory::from(zbytes);
    let zmachine = ZMachine::from_memory(memory, &config);

    let expected_file_length = zmachine.memory.borrow().file_length();
    if expected_file_length != 0 && expected_file_length > bytes_read as u32 {
      // It's okay for the memory too be too big, but not too small.
      return Err(Error::ZFileTooShort);
    }

    Ok(zmachine)
  }

  fn reset_interpreter_flags(&mut self) {
    // The interpreter sets flags in the header to express its capabilities to the game.
    let flag1_mask = !0b01110000;  // no status line, no split screen, fixed-width font
    let old_val = self.memory.borrow().flag1();
    self.memory.borrow_mut().set_flag1(old_val & flag1_mask);
  }

  pub fn init_windows(&mut self) {
    initscr();
    raw();
    noecho();
    refresh();

    getmaxyx(stdscr(), &mut self.num_rows, &mut self.num_cols);

    let win = newwin(1, self.num_cols, 0, 0);
    scrollok(win, false);
    wattron(win, A_REVERSE());
    self.status_window = Some(win);
    wrefresh(win);

    let win = newwin(self.num_rows - 1, self.num_cols, 1, 0);
    scrollok(win, true);
    wmove(win, self.num_rows - 2, 0);
    self.main_window = Some(win);
    wrefresh(win);
  }

  pub fn tear_down_windows(&mut self) {
    self.status_window = None;
    self.main_window = None;
    endwin();
  }

  pub fn run(&mut self, start_pc: Option<usize>) -> Result<()> {
    start_pc.map(|new_pc| self.pc.set_raw_pc(new_pc));
    self.init_windows();

    // TODO: check version number
    loop {
      let r = self.process_opcode();
      match r {
        Err(Error::Quitting) => break,
        Err(Error::Restart) => {
          self.memory.borrow_mut().restore_dynamic_bytes();
          self.pc.set_pc(self.memory.borrow().starting_pc());
        }
        Err(_) => return r,
        _ => {}
      }
    }

    self.tear_down_windows();
    Ok(())
  }

  fn process_opcode(&mut self) -> Result<()> {
    info!(target: "pctrace", "PC: {}", usize::from(self.pc.pc()));

    let first_byte = self.read_pc_byte();
    let top_two_bits = first_byte & 0b11000000;

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
      let operands = self.read_var_operands();
      self.dispatch_2op(start_pc, opcode_number, operands)
    } else {
      let operands = self.read_var_operands();
      match opcode_number {
        0x00 => ops::varops::call_0x00(self, operands),
        0x01 => ops::varops::storew_0x01(self, operands),
        0x02 => ops::varops::storeb_0x02(self, operands),
        0x03 => ops::varops::put_prop_0x03(self, operands),
        0x04 => ops::varops::read_0x04(self, operands),
        0x05 => ops::varops::print_char_0x05(self, operands),
        0x06 => ops::varops::print_num_0x06(self, operands),
        0x07 => {
          let encoded = self.read_pc_byte();
          let variable = VariableRef::decode(encoded);
          ops::varops::random_0x07(self, operands, variable)
        }
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
      0x01 => ops::zeroops::rfalse_0x01(self),
      0x02 => ops::zeroops::print_0x02(self),
      0x03 => ops::zeroops::print_ret_0x03(self),
      0x04 => ops::zeroops::nop_0x04(self),
      0x05 => ops::zeroops::save_0x05(self),
      0x07 => ops::zeroops::restart_0x07(self),
      0x08 => ops::zeroops::ret_popped_0x08(self),
      0x09 => ops::zeroops::pop_0x09(self),
      0x0a => ops::zeroops::quit_0x0a(self),
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
      0x01 => self.process_1op_with_return(operand, &ops::oneops::get_sibling_0x01),
      0x02 => self.process_1op_with_return(operand, &ops::oneops::get_child_0x02),
      0x03 => self.process_1op_with_return(operand, &ops::oneops::get_parent_0x03),
      0x04 => self.process_1op_with_return(operand, &ops::oneops::get_prop_len_0x04),
      0x05 => ops::oneops::inc_0x05(self, operand),
      0x06 => ops::oneops::dec_0x06(self, operand),
      0x07 => ops::oneops::print_addr_0x07(self, operand),
      0x09 => ops::oneops::remove_obj_0x09(self, operand),
      0x0a => ops::oneops::print_obj_0x0a(self, operand),
      0x0b => ops::oneops::ret_0x0b(self, operand),
      0x0c => ops::oneops::jump_0x0c(self, operand),
      0x0d => ops::oneops::print_paddr_0x0d(self, operand),
      0x0e => self.process_1op_with_return(operand, &ops::oneops::load_0x0e),
      _ => {
        panic!("Unknown short 1op opcode: {:#x} @{:#x}", op, start_pc);
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
    let operands = [first, second, Operand::Omitted, Operand::Omitted];
    self.dispatch_2op(start_pc, opcode_number, operands)
  }

  fn dispatch_basic_2op(&mut self,
                        operands: [Operand; 4],
                        op_func: &Fn(&mut Self, Operand, Operand) -> Result<()>)
                        -> Result<()> {
    let lhs = operands[0];
    let rhs = operands[1];
    // TODO: add checking that there are no extra operands.
    op_func(self, lhs, rhs)
  }

  fn dispatch_2op_with_return(&mut self,
                              operands: [Operand; 4],
                              op_func: &Fn(&mut Self, Operand, Operand, VariableRef) -> Result<()>)
                              -> Result<()> {
    let encoded = self.read_pc_byte();
    let variable = VariableRef::decode(encoded);
    // TODO: add checking that there are no extra operands.
    op_func(self, operands[0], operands[1], variable)
  }

  fn dispatch_2op(&mut self, start_pc: usize, opcode: u8, operands: [Operand; 4]) -> Result<()> {
    match opcode {
      0x01 => ops::twoops::je_0x01(self, operands),
      0x02 => self.dispatch_basic_2op(operands, &ops::twoops::jl_0x02),
      0x03 => self.dispatch_basic_2op(operands, &ops::twoops::jg_0x03),
      0x04 => self.dispatch_basic_2op(operands, &ops::twoops::dec_chk_0x04),
      0x05 => self.dispatch_basic_2op(operands, &ops::twoops::inc_chk_0x05),
      0x06 => self.dispatch_basic_2op(operands, &ops::twoops::jin_0x06),
      0x07 => self.dispatch_basic_2op(operands, &ops::twoops::test_0x07),
      0x08 => self.dispatch_2op_with_return(operands, &ops::twoops::or_0x08),
      0x09 => self.dispatch_2op_with_return(operands, &ops::twoops::and_0x09),
      0x0a => self.dispatch_basic_2op(operands, &ops::twoops::test_attr_0x0a),
      0x0b => self.dispatch_basic_2op(operands, &ops::twoops::set_attr_0x0b),
      0x0c => self.dispatch_basic_2op(operands, &ops::twoops::clear_attr_0x0c),
      0x0d => self.dispatch_basic_2op(operands, &ops::twoops::store_0x0d),
      0x0e => self.dispatch_basic_2op(operands, &ops::twoops::insert_obj_0x0e),
      0x0f => self.dispatch_2op_with_return(operands, &ops::twoops::loadw_0x0f),
      0x10 => self.dispatch_2op_with_return(operands, &ops::twoops::loadb_0x10),
      0x11 => self.dispatch_2op_with_return(operands, &ops::twoops::get_prop_0x11),
      0x12 => self.dispatch_2op_with_return(operands, &ops::twoops::get_prop_addr_0x12),
      0x13 => self.dispatch_2op_with_return(operands, &ops::twoops::get_next_prop_0x13),
      0x14 => self.dispatch_2op_with_return(operands, &ops::twoops::add_0x14),
      0x15 => self.dispatch_2op_with_return(operands, &ops::twoops::sub_0x15),
      0x16 => self.dispatch_2op_with_return(operands, &ops::twoops::mul_0x16),
      0x17 => self.dispatch_2op_with_return(operands, &ops::twoops::div_0x17),
      0x18 => self.dispatch_2op_with_return(operands, &ops::twoops::mod_0x18),

      _ => panic!("Unknown long opcode: {:#x} @{:#x}", opcode, start_pc),
    }
  }
}

impl VM for ZMachine {
  type ObjTable = MemoryMappedObjectTable;
  type PropertyTable = MemoryMappedPropertyTable;

  fn read_pc_byte(&mut self) -> u8 {
    self.pc.next_byte()
  }

  fn read_pc_word(&mut self) -> u16 {
    self.pc.next_word()
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
    Ok(self.memory.borrow().read_global(global_idx))
  }

  fn write_global(&mut self, global_idx: u8, val: u16) -> Result<()> {
    self.memory.borrow_mut().write_global(global_idx, val);
    Ok(())
  }

  fn read_memory<T>(&self, ptr: T) -> Result<u16>
    where T: Into<RawPtr> {
    Ok(self.memory.borrow().u16_at(ptr))
  }

  fn write_memory<T>(&mut self, ptr: T, val: u16) -> Result<()>
    where T: Into<RawPtr> {
    self.memory.borrow_mut().set_u16_at(val, ptr);
    Ok(())
  }

  fn read_memory_u8<T>(&self, ptr: T) -> Result<u8>
    where T: Into<RawPtr> {
    Ok(self.memory.borrow().u8_at(ptr))
  }

  fn write_memory_u8<T>(&mut self, ptr: T, val: u8) -> Result<()>
    where T: Into<RawPtr> {
    self.memory.borrow_mut().set_u8_at(val, ptr);
    Ok(())
  }

  fn object_table(&self) -> Result<MemoryMappedObjectTable> {
    let ptr = self.memory.borrow().property_table_ptr();
    Ok(MemoryMappedObjectTable::new(ptr, self.memory.clone()))
  }

  fn num_dict_entries(&self) -> u16 {
    self.dict.num_entries()
  }

  fn dict_entry(&self, number: u16) -> BytePtr {
    self.dict.entry_ptr(number)
  }

  fn rand(&mut self, range: u16) -> u16 {
    let range = range as i16;
    if range < 0 {
      // Seed with -range.
      let new_seed = [-range as usize];
      self.rng.reseed(&new_seed);
      0
    } else if range == 0 {
      // Seed as randomly as possible.
      self.rng = StdRng::new().unwrap();
      0
    } else {
      // Return a random number in 1..range.
      (self.rng.next_u32() as u16) % range as u16 + 1
    }
  }

  fn write_status_line(&self, str: &str) {
    self.status_window.map(|w| {
      self.main_window.map(|mw| {
        let mut x = 0i32;
        let mut y = 0i32;
        getyx(mw, &mut y, &mut x);

        mvwprintw(w, 0, 0, str);
        wmove(mw, y, x);
        wrefresh(w);
        wrefresh(mw);
      })
    });
  }

  fn write_main_window_char(&self, ch: u16) {
    self.main_window.map(|w| {
      waddch(w, ch as u32);
      wrefresh(w);
    });
  }

  fn write_main_window(&self, str: &str) {
    self.main_window.map(|w| {
      wprintw(w, str);
      wrefresh(w);
    });
  }

  fn read_line(&self) -> Result<String> {
    self.main_window
      .map(|w| {
        let rl = ZReadline::new(w);
        Ok(rl.readline())
      })
      .unwrap()
  }

  fn screen_width(&self) -> u16 {
    self.num_cols as u16
  }

  fn save(&self) -> Result<()> {
    let bytes = Quetzal::write(&self.memory.borrow(), &self.stack, &self.pc)?;
    // TODO: make this sane.
    let mut f = File::create("foobar.zinf").unwrap();
    f.write_all(&bytes).unwrap();
    Ok(())
  }

  fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> Result<WordPtr> {
    let abbrev_table_ptr = self.memory.borrow().abbrev_table_ptr();
    let abbrev_entry_ptr =
      abbrev_table_ptr.inc_by((32 * (abbrev_table as u16 - 1) + abbrev_index as u16) * 2);
    let abbrev_addr = self.memory.borrow().u16_at(abbrev_entry_ptr);
    Ok(WordPtr::new(abbrev_addr))
  }
}
