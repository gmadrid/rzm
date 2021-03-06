use std::cell::RefCell;
use std::rc::Rc;
use super::memory::Memory;
use super::ptrs::RawPtr;

pub struct PC {
  pc: RawPtr,
  memory: Rc<RefCell<Memory>>,
}

impl PC {
  pub fn new<P>(p: P, memory: Rc<RefCell<Memory>>) -> PC
    where P: Into<RawPtr> {
    PC {
      pc: p.into(),
      memory: memory,
    }
  }

  pub fn pc(&self) -> RawPtr {
    self.pc
  }

  pub fn set_pc<P>(&mut self, p: P)
    where P: Into<RawPtr> {
    self.pc = p.into();
  }

  pub fn set_raw_pc(&mut self, p: usize) {
    self.pc = RawPtr::new(p);
  }

  pub fn next_byte(&mut self) -> u8 {
    let result = self.memory.borrow().u8_at(self.pc);
    self.pc.inc_by(1usize);
    result
  }

  pub fn next_word(&mut self) -> u16 {
    let result = self.memory.borrow().u16_at(self.pc);
    self.pc.inc_by(2usize);
    result
  }
}

#[cfg(test)]
mod test {
  use super::PC;
  use super::super::{BytePtr, PackedAddr};
  use super::super::memory::Memory;

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
