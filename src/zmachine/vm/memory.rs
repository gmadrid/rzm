// YES

use byteorder::{BigEndian, ByteOrder};
use super::ptrs::{BytePtr, RawPtr};

const FLAG1_INDEX: u16 = 0x01;
const STARTING_PC_INDEX: u16 = 0x06;
const PROPERTY_TABLE_INDEX: u16 = 0x0a;
const GLOBAL_TABLE_INDEX: u16 = 0x0c;
const ABBREV_TABLE_INDEX: u16 = 0x18;
const FILE_LENGTH_INDEX: u16 = 0x1a;

pub struct Memory {
  bytes: Vec<u8>,
}

impl From<Vec<u8>> for Memory {
  fn from(vec: Vec<u8>) -> Memory {
    Memory::new(vec)
  }
}

impl Memory {
  fn new(bytes: Vec<u8>) -> Memory {
    Memory { bytes: bytes }
  }

  pub fn u8_at<P>(&self, ptr: P) -> u8
    where P: Into<RawPtr> {
    self.bytes[ptr.into().ptr()]
  }

  pub fn set_u8_at<P>(&mut self, val: u8, ptr: P)
    where P: Into<RawPtr> {
    self.bytes[ptr.into().ptr()] = val;
  }

  pub fn u16_at<P>(&self, ptr: P) -> u16
    where P: Into<RawPtr> {
    BigEndian::read_u16(&self.bytes[ptr.into().ptr()..])
  }

  pub fn set_u16_at<P>(&mut self, val: u16, ptr: P)
    where P: Into<RawPtr> {
    BigEndian::write_u16(&mut self.bytes[ptr.into().ptr()..], val);
  }

  pub fn u32_at_index(&self, index: usize) -> u32 {
    // TODO: test this.
    BigEndian::read_u32(&self.bytes[index..])
  }

  pub fn flag1(&self) -> u8 {
    self.u8_at(BytePtr::new(FLAG1_INDEX))
  }

  pub fn set_flag1(&mut self, val: u8) {
    self.set_u8_at(val, BytePtr::new(FLAG1_INDEX));
  }

  pub fn file_length(&self) -> u32 {
    self.u16_at(BytePtr::new(FILE_LENGTH_INDEX)) as u32 * 2
  }

  pub fn starting_pc(&self) -> BytePtr {
    BytePtr::new(self.u16_at(BytePtr::new(STARTING_PC_INDEX)))
  }

  pub fn property_table_ptr(&self) -> BytePtr {
    BytePtr::new(self.u16_at(BytePtr::new(PROPERTY_TABLE_INDEX)))
  }

  pub fn abbrev_table_ptr(&self) -> BytePtr {
    BytePtr::new(self.u16_at(BytePtr::new(ABBREV_TABLE_INDEX)))
  }

  pub fn global_base_ptr(&self) -> BytePtr {
    BytePtr::new(self.u16_at(BytePtr::new(GLOBAL_TABLE_INDEX)))
  }

  fn global_ptr(&self, global_idx: u8) -> BytePtr {
    assert!(global_idx < 240, "Max global is 239: {}", global_idx);
    let base = self.global_base_ptr();
    base.inc_by(global_idx as u16 * 2)
  }

  pub fn read_global(&self, global_idx: u8) -> u16 {
    let ptr = self.global_ptr(global_idx);
    self.u16_at(ptr)
    //    BigEndian::read_u16(&self.bytes[offset..])
  }

  pub fn write_global(&mut self, global_idx: u8, val: u16) {
    let ptr = self.global_ptr(global_idx);
    self.set_u16_at(val, ptr);
    //    BigEndian::write_u16(&mut self.bytes[offset..], val);
  }
}

#[cfg(test)]
mod test {
  use byteorder::{BigEndian, ByteOrder};
  use super::Memory;
  use super::super::ptrs::{BytePtr, RawPtr};

  #[test]
  fn test_from() {
    let memory = Memory::from(vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(6, memory.bytes.len());
  }

  #[test]
  fn test_memory() {
    let mut memory: Memory = From::from(vec![0, 1, 2, 3, 4, 5]);

    assert_eq!(2, memory.u8_at(BytePtr::new(2)));
    assert_eq!(3, memory.u8_at(BytePtr::new(3)));
    assert_eq!(5, memory.u8_at(BytePtr::new(5)));

    assert_eq!(0x0102, memory.u16_at(BytePtr::new(1)));
    assert_eq!(0x0405, memory.u16_at(BytePtr::new(4)));

    memory.set_u8_at(8, BytePtr::new(4));
    assert_eq!(0x0805, memory.u16_at(BytePtr::new(4)));
  }

  #[test]
  fn test_globals() {
    // 608 = 0x80 (global base) + 2 * 0xf0 (number of globals)
    let mut memory = Memory::from(vec![0; 608]);

    // Set up the memory so that the global table is at 0x80 and has
    // the value 0x84 at global 2 (0x84)
    let global_offset = 0x80usize;
    let val = 0x94u16;
    BigEndian::write_u16(&mut memory.bytes[super::GLOBAL_TABLE_INDEX as usize..],
                         global_offset as u16);
    BigEndian::write_u16(&mut memory.bytes[global_offset + 2 * 2..], val);

    assert_eq!(val, memory.read_global(2));

    memory.write_global(0, 0x0809);
    assert_eq!(0x0809, memory.read_global(0));

    memory.write_global(239, 0x0708);
    assert_eq!(0x0708, memory.read_global(239));
  }

  #[test]
  #[should_panic]
  fn test_globals_overflow_read() {
    // 608 = 0x80 (global base) + 2 * 0xf0 (number of globals)
    let memory = Memory::from(vec![0; 608]);

    memory.read_global(240);
  }

  #[test]
  #[should_panic]
  fn test_globals_overflow_write() {
    // 608 = 0x80 (global base) + 2 * 0xf0 (number of globals)
    let mut memory = Memory::from(vec![0; 608]);

    memory.write_global(240, 0);
  }
}
