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

  pub fn next_byte(&mut self, memory: &Memory) -> u8 {
    let result = memory.u8_at_index(self.pc);
    self.pc += 1;
    result
  }

  pub fn next_word(&mut self, memory: &Memory) -> u16 {
    let result = memory.u16_at_index(self.pc);
    self.pc += 2;
    result
  }
}

#[cfg(test)]
mod test {
  use super::PC;
  use super::super::memory::Memory;

  #[test]
  fn test_pc() {
    let mut pc = PC::new(54);
    assert_eq!(54, pc.pc());
    pc.set_pc(88);
    assert_eq!(88, pc.pc());
    pc.set_pc_to_packed_addr(64);
    assert_eq!(128, pc.pc());
  }

  #[test]
  fn test_memory() {
    let memory: Memory = From::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let mut pc = PC::new(2);
    assert_eq!(2, pc.next_byte(&memory));
    assert_eq!(3, pc.pc());

    assert_eq!(0x0304, pc.next_word(&memory));
    assert_eq!(5, pc.pc());
  }
}
