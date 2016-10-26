use result::{Error, Result};
use super::memory::Memory;

pub struct PC {
  pc: usize,
}

impl PC {
  pub fn new(pc: usize) -> PC {
    PC { pc: pc }
  }

  pub fn pc(&self) -> usize {
    self.pc
  }

  pub fn set_pc(&mut self, pc: usize) {
    self.pc = pc;
  }

  pub fn set_pc_to_packed_addr(&mut self, packed_addr: usize) {
    self.set_pc(packed_addr * 2);
  }

  pub fn next_byte(&mut self, memory: &Memory) -> Result<u8> {
    let result = memory.u8_at_index(self.pc);
    self.pc += 1;
    Ok(result)
  }

  pub fn next_word(&mut self, memory: &Memory) -> Result<u16> {
    let result = memory.u16_at_index(self.pc);
    self.pc += 2;
    Ok(result)
  }
}
