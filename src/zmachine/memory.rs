use byteorder::{BigEndian, ByteOrder};

const FLAG1_INDEX: usize = 0x01;
const STARTING_PC_INDEX: usize = 0x06;
const FILE_LENGTH_INDEX: usize = 0x1a;

pub struct Memory {
  bytes: Vec<u8>,
}

impl Memory {
  fn new(bytes: Vec<u8>) -> Memory {
    Memory { bytes: bytes }

    // zmachine.hmem_index = BigEndian::read_u16(&zmachine.header[0x04..]);
    // zmachine.smem_index = BigEndian::read_u16(&zmachine.header[0x0e..]);
  }

  pub fn u8_at_index(&self, index: usize) -> u8 {
    self.bytes[index]
  }

  pub fn set_u8_at_index(&mut self, val: u8, index: usize) {
    self.bytes[index] = val;
  }

  pub fn u16_at_index(&self, index: usize) -> u16 {
    BigEndian::read_u16(&self.bytes[index..])
  }

  pub fn flag1(&self) -> u8 {
    self.u8_at_index(FLAG1_INDEX)
  }

  pub fn set_flag1(&mut self, val: u8) {
    self.set_u8_at_index(val, FLAG1_INDEX);
  }

  pub fn file_length(&self) -> u32 {
    self.u16_at_index(FILE_LENGTH_INDEX) as u32 * 2
  }

  pub fn starting_pc(&self) -> usize {
    self.u16_at_index(STARTING_PC_INDEX) as usize
  }
}

impl From<Vec<u8>> for Memory {
  fn from(vec: Vec<u8>) -> Memory {
    Memory::new(vec)
  }
}
