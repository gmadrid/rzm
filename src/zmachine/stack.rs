use byteorder::{BigEndian, ByteOrder};
use std::u16;
use super::opcodes::{Operand, Operands, Operation};

pub struct Stack {
  stack: Vec<u8>,
  sp: usize, // index of next empty location in stack
  fp: usize, // index of base of current frame (the saved fp)
  base_sp: usize, // index of bottom of current frame's stack
}

// Each stack frame:
//    +------------------------------------------------------+
//    | 0x00: fp (u16) to previous frame                     |
//    | 0x02: pc (u32) to next instruction in previous frame |
//    | 0x06: num locals (u8)                                |
//    | 0x07: result location (u8)                           |
//    | 0x08: L1                                             |
//    |  ...  LN                                             |
//    |       base of this frame's stack                     |
//    |                                                      |
//    |                                                      |
//    |                                                      |
//
impl Stack {
  pub fn new(size: usize) -> Stack {
    assert!(size < u16::MAX as usize, "Cannot to stack size > 0xffff");

    let mut stack = Stack {
      stack: Vec::with_capacity(size),
      sp: 0,
      fp: 0,
      base_sp: 0,
    };
    stack.stack.resize(size, 0);

    // Initialize the base stack frame.
    stack.push_u16(0);
    stack.push_u32(0);
    stack.push_u8(0);
    stack.push_u8(0);
    stack.base_sp = stack.sp;
    stack
  }

  pub fn new_frame(&mut self,
                   pc: usize,
                   num_locals: u8,
                   result_location: u8,
                   operands: &[Operand]) {
    let new_fp = self.sp;
    let old_fp = self.fp;
    self.push_u16(old_fp as u16);
    self.push_u32(pc as u32);
    self.push_u8(num_locals);
    self.push_u8(result_location);
    for _ in 0..num_locals {
      self.push_u16(0);
    }

    for (i, operand) in operands.iter().enumerate() {
      if i >= num_locals as usize || Operand::Omitted == *operand {
        break;
      }
    }

    self.fp = new_fp;
    self.base_sp = self.sp;
  }

  fn push_u32(&mut self, val: u32) {
    let mut slice = self.stack.as_mut_slice();
    BigEndian::write_u32(&mut slice[self.sp..], val);
    self.sp += 4;
  }

  fn push_u16(&mut self, val: u16) {
    let mut slice = self.stack.as_mut_slice();
    BigEndian::write_u16(&mut slice[self.sp..], val);
    self.sp += 2;
  }

  fn push_u8(&mut self, val: u8) {
    self.stack[self.sp] = val;
    self.sp += 1;
  }
}

#[cfg(test)]
mod test {
  use std::u16;
  use super::Stack;

  #[test]
  fn test_capacity() {
    let stack = Stack::new(100);
    assert_eq!(100, stack.stack.capacity());
    assert_eq!(100, stack.stack.len());

    let mut stack = Stack::new(256);
    assert_eq!(256, stack.stack.capacity());
    assert_eq!(256, stack.stack.len());

    // We should be able to push 256 items into this stack with no error.
    for i in 0..((255 - stack.sp) as u8) {
      stack.push_u8(i);
    }
  }

  #[test]
  #[should_panic]
  fn test_overflow() {
    let mut stack = Stack::new(100);

    // Push one too many.
    for i in 0..((101 - stack.sp) as u8) {
      stack.push_u8(i);
    }
  }

  #[test]
  #[should_panic]
  fn test_size_restriction() {
    Stack::new(u16::MAX as usize + 1);
  }

  #[test]
  fn test_push_u8() {
    let mut stack = Stack::new(100);
    stack.push_u8(5);
    stack.push_u8(10);
    stack.push_u8(15);
    assert_eq!(&stack.stack[stack.base_sp..stack.base_sp + 3],
               vec![5, 10, 15].as_slice());
  }

  #[test]
  fn test_push_u16() {
    let mut stack = Stack::new(100);
    stack.push_u16(5);
    stack.push_u16(10);
    stack.push_u16(15);
    assert_eq!(&stack.stack[stack.base_sp..stack.base_sp + 6],
               vec![0, 5, 0, 10, 0, 15].as_slice());
  }

  #[test]
  fn test_push_u32() {
    let mut stack = Stack::new(100);
    stack.push_u32(5);
    stack.push_u32(10);
    stack.push_u32(15);
    assert_eq!(&stack.stack[stack.base_sp..stack.base_sp + 12],
               vec![0, 0, 0, 5, 0, 0, 0, 10, 0, 0, 0, 15].as_slice());
  }

  #[test]
  fn test_push_mixed() {
    let mut stack = Stack::new(100);
    stack.push_u8(5);
    stack.push_u16(10);
    stack.push_u32(15);
    stack.push_u32(0xfedcba90);
    stack.push_u16(20);
    stack.push_u8(25);
    assert_eq!(&stack.stack[stack.base_sp..stack.base_sp + 14],
               vec![5, 0, 10, 0, 0, 0, 15, 0xfe, 0xdc, 0xba, 0x90, 0, 20, 25].as_slice());
  }
}
