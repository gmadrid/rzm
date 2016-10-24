use super::memory::Memory;

const STARTING_PC_INDEX: usize = 0x06;

pub struct Header<'a> {
  memory: &'a Memory,
}

impl<'a> Header<'a> {
  fn new(memory: &'a Memory) -> Header<'a> {
    Header { memory: memory }
  }

  pub fn starting_pc(&self) -> usize {
    self.memory.u16_at_index(STARTING_PC_INDEX) as usize
  }
}

impl<'a> From<&'a Memory> for Header<'a> {
  fn from(memory: &Memory) -> Header {
    Header::new(memory)
  }
}
