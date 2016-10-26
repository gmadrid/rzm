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
  }

  pub fn size(&self) -> usize {
    self.bytes.len()
  }

  pub fn u8_at_index(&self, index: usize) -> u8 {
    self.bytes[index]
  }

  pub fn set_index_to_u8(&mut self, index: usize, val: u8) {
    self.bytes[index] = val;
  }

  pub fn u16_at_index(&self, index: usize) -> u16 {
    BigEndian::read_u16(&self.bytes[index..])
  }

  pub fn flag1(&self) -> u8 {
    self.u8_at_index(FLAG1_INDEX)
  }

  pub fn set_flag1(&mut self, val: u8) {
    self.set_index_to_u8(FLAG1_INDEX, val);
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

#[test]
fn test_from() {
  let memory: Memory = From::from(vec![0, 1, 2, 3, 4, 5]);
  assert_eq!(6, memory.size());
}

#[test]
fn test_memory() {
  let mut memory: Memory = From::from(vec![0, 1, 2, 3, 4, 5]);
  assert_eq!(6, memory.size());

  assert_eq!(2, memory.u8_at_index(2));
  assert_eq!(3, memory.u8_at_index(3));
  assert_eq!(5, memory.u8_at_index(5));

  assert_eq!(0x0102, memory.u16_at_index(1));
  assert_eq!(0x0405, memory.u16_at_index(4));

  memory.set_index_to_u8(4, 8);
  assert_eq!(0x0805, memory.u16_at_index(4));
}
