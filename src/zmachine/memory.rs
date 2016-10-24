use byteorder::{BigEndian, ByteOrder};

pub struct Memory {
  bytes: Vec<u8>,
}

impl Memory {
  fn new(bytes: Vec<u8>) -> Memory {
    Memory { bytes: bytes }
  }

  pub fn u8_at_index(&self, index: usize) -> u8 {
    //    self.bytes[index]
    32
  }

  pub fn u16_at_index(&self, index: usize) -> u16 {
    //    BigEndian::read_u16(&self.bytes[index..])
    33
  }
}

impl From<Vec<u8>> for Memory {
  fn from(vec: Vec<u8>) -> Memory {
    Memory::new(vec)
  }
}
