use byteorder::{BigEndian, ByteOrder};

const FLAG1_INDEX: usize = 0x01;
const STARTING_PC_INDEX: usize = 0x06;
const PROPERTY_TABLE_INDEX: usize = 0x0a;
const GLOBAL_TABLE_INDEX: usize = 0x0c;
const ABBREV_TABLE_INDEX: usize = 0x18;
const FILE_LENGTH_INDEX: usize = 0x1a;

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

  #[cfg(test)]
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


  pub fn set_u16_at_index(&mut self, index: usize, val: u16) {
    BigEndian::write_u16(&mut self.bytes[index..], val);
  }

  pub fn u32_at_index(&self, index: usize) -> u32 {
    // TODO: test this.
    BigEndian::read_u32(&self.bytes[index..])
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

  pub fn property_table_offset(&self) -> usize {
    self.u16_at_index(PROPERTY_TABLE_INDEX) as usize
  }

  pub fn abbrev_table_offset(&self) -> usize {
    self.u16_at_index(ABBREV_TABLE_INDEX) as usize
  }

  pub fn global_base_byteaddress(&self) -> usize {
    self.u16_at_index(GLOBAL_TABLE_INDEX) as usize
  }

  fn global_offset(&self, global_idx: u8) -> usize {
    assert!(global_idx < 240, "Max global is 239: {}", global_idx);
    let base = self.global_base_byteaddress();
    base + global_idx as usize * 2
  }

  pub fn read_global(&self, global_idx: u8) -> u16 {
    let offset = self.global_offset(global_idx);
    BigEndian::read_u16(&self.bytes[offset..])
  }

  pub fn write_global(&mut self, global_idx: u8, val: u16) {
    let offset = self.global_offset(global_idx);
    BigEndian::write_u16(&mut self.bytes[offset..], val);
  }
}

#[cfg(test)]
mod test {
  use byteorder::{BigEndian, ByteOrder};
  use super::Memory;

  #[test]
  fn test_from() {
    let memory = Memory::from(vec![0, 1, 2, 3, 4, 5]);
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

  #[test]
  fn test_globals() {
    // 608 = 0x80 (global base) + 2 * 0xf0 (number of globals)
    let mut memory = Memory::from(vec![0; 608]);

    // Set up the memory so that the global table is at 0x80 and has
    // the value 0x84 at global 2 (0x84)
    let global_offset = 0x80usize;
    let val = 0x94u16;
    BigEndian::write_u16(&mut memory.bytes[super::GLOBAL_TABLE_INDEX..],
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
