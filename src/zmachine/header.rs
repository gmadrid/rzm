use byteorder::{BigEndian, ByteOrder};
use super::memory::Memory;

const STARTING_PC_INDEX: usize = 0x06;

pub struct Header<'a> {
  memory: &'a Memory,
}

impl<'a> Header<'a> {
  pub fn new(memory: &'a Memory) -> Header<'a> {
    Header { memory: memory }
  }

  pub fn starting_pc(&self) -> usize {
    self.memory.u16_at_index(STARTING_PC_INDEX) as usize
  }
}
