use super::memory::Memory;

pub struct PC<'a> {
  pc: usize,
  memory: &'a Memory,
}

impl<'a> PC<'a> {
  pub fn new(pc: usize, memory: &'a Memory) -> PC<'a> {
    PC {
      pc: pc,
      memory: memory,
    }
  }
}
