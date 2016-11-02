use super::vm::Memory;
use super::vm::RawPtr;

pub struct PC {
  pc: RawPtr,
}

impl PC {
  pub fn new<P>(p: P) -> PC
    where P: Into<RawPtr> {
    PC { pc: p.into() }
  }

  pub fn pc(&self) -> RawPtr {
    self.pc
  }

  pub fn set_pc<P>(&mut self, p: P)
    where P: Into<RawPtr> {
    self.pc = p.into();
  }

  pub fn next_byte(&mut self, memory: &Memory) -> u8 {
    let result = memory.u8_at(self.pc);
    self.pc.inc_by(1usize);
    result
  }

  pub fn next_word(&mut self, memory: &Memory) -> u16 {
    let result = memory.u16_at(self.pc);
    self.pc.inc_by(2usize);
    result
  }
}

#[cfg(test)]
mod test {
  use super::PC;
  use super::super::vm::{BytePtr, PackedAddr};
  use super::super::vm::Memory;

  #[test]
  fn test_pc() {
    let mut pc = PC::new(BytePtr::new(54));
    assert_eq!(54, pc.pc().ptr());
    pc.set_pc(BytePtr::new(88));
    assert_eq!(88, pc.pc().ptr());
    pc.set_pc(PackedAddr::new(64));
    assert_eq!(128, pc.pc().ptr());
  }

  #[test]
  fn test_memory() {
    let memory: Memory = From::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let mut pc = PC::new(BytePtr::new(2));
    assert_eq!(2, pc.next_byte(&memory));
    assert_eq!(3, pc.pc().ptr());

    assert_eq!(0x0304, pc.next_word(&memory));
    assert_eq!(5, pc.pc().ptr());
  }
}
