use byteorder::{BigEndian, ByteOrder};
use result::Result;
use std::u16;
use super::vm::VariableRef;

// Each stack frame:
//    +------------------------------------------------------+
//    | 0x00: fp (u16) to previous frame                     |
//    | 0x02: pc (u32) to next instruction in previous frame |
//    | 0x06: num locals (u8)                                |
//    | 0x07: result location (u8)                           |
//    | 0x08: L0 (two bytes each)                            |
//    |  ...  LN                                             |
//    |       base of this frame's stack                     |
//    |                                                      |
//    |                                                      |
//    |                                                      |
//
const PC_OFFSET: usize = 0x02;
const NUM_LOCALS_OFFSET: usize = 0x06;
const RESULT_LOCATION_OFFSET: usize = 0x07;
const FIRST_LOCAL_OFFSET: usize = 0x08;

pub struct Stack {
  stack: Vec<u8>,
  sp: usize, // index of next empty location in stack
  fp: usize, // index of base of current frame (the saved fp)
  base_sp: usize, // index of bottom of current frame's stack
}

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

  pub fn map_frames<T>(&self, mut f: T) -> Result<()>
    where T: FnMut(u32, u8, u8, Vec<u16>, Vec<u16>) -> Result<()> {
    for (start_of_frame, end_of_frame) in self.frame_ptrs() {
      let return_pc = BigEndian::read_u32(&self.stack[start_of_frame + PC_OFFSET..]);

      let p_flag = 0u8;  // No Call_N implemented for v3.
      let num_locals = self.stack[start_of_frame + NUM_LOCALS_OFFSET];
      let flags = (p_flag << 4) + (num_locals & 0x0f);

      let encoded_result_variable = self.stack[start_of_frame + RESULT_LOCATION_OFFSET];

      let start_of_eval_stack = start_of_frame + FIRST_LOCAL_OFFSET + 2 * num_locals as usize;
      let eval_stack_words = ((end_of_frame - start_of_eval_stack) / 2) as u16;

      // TODO: save/write num_args_passed

      let mut locals = Vec::<u16>::new();
      let local_offset = start_of_frame + FIRST_LOCAL_OFFSET;
      for idx in 0..num_locals {
        let local = BigEndian::read_u16(&self.stack[local_offset + 2 * idx as usize..]);
        locals.push(local);
      }

      let mut stack_words = Vec::<u16>::new();
      for idx in 0..eval_stack_words {
        let stack_word = BigEndian::read_u16(&self.stack[start_of_eval_stack + 2 * idx as usize..]);
        stack_words.push(stack_word);
      }

      f(return_pc,
        flags,
        encoded_result_variable,
        locals,
        stack_words)?;
    }
    Ok(())
  }

  // Return a vector of (start_of_frame, end_of_frame) pairs.
  // start_of_frame is the offset of the beginning of the frame as described
  // above. end_of_frame is the offset of the first byte in the stack *after*
  // the frame.
  fn frame_ptrs(&self) -> Vec<(usize, usize)> {
    let mut ptrs = Vec::<(usize, usize)>::new();
    let mut start_of_frame = self.fp;
    let mut end_of_frame = self.sp;

    loop {
      ptrs.push((start_of_frame, end_of_frame));
      let next_fp = BigEndian::read_u16(&self.stack[start_of_frame..]) as usize;
      if start_of_frame == 0 {
        break;
      }
      end_of_frame = start_of_frame;
      start_of_frame = next_fp;
    }
    ptrs.reverse();
    ptrs
  }

  // Allocate a new stack frame, adding it to the call stack.
  // Also allocate space for local variables, setting them all to zero.
  pub fn new_frame(&mut self, pc: usize, num_locals: u8, result_location: VariableRef) {
    let new_fp = self.sp;
    let old_fp = self.fp;
    self.push_u16(old_fp as u16);
    self.push_u32(pc as u32);
    self.push_u8(num_locals);
    self.push_u8(VariableRef::encode(result_location));
    for _ in 0..num_locals {
      self.push_u16(0);
    }

    self.fp = new_fp;
    self.base_sp = self.sp;
  }

  pub fn pop_frame(&mut self) -> (usize, VariableRef) {
    // Read these values before resetting the fp.
    let old_fp = BigEndian::read_u16(&self.stack[self.fp..]);
    let old_pc = BigEndian::read_u32(&self.stack[self.fp + PC_OFFSET..]);
    let old_sp = self.fp;
    let return_var = VariableRef::decode(self.stack[self.fp + RESULT_LOCATION_OFFSET]);

    self.fp = old_fp as usize;
    self.sp = old_sp;

    // Need to get the number of locals in the new frame to reset the base_sp.
    let num_locals = self.stack[self.fp + NUM_LOCALS_OFFSET];
    self.base_sp = self.fp + FIRST_LOCAL_OFFSET + 2 * num_locals as usize;

    // Return the old pc value and the result location.
    (old_pc as usize, return_var)
  }

  fn offset_for_local(&self, local_idx: u8) -> usize {
    let num_locals = self.stack[self.fp + NUM_LOCALS_OFFSET];
    assert!(local_idx < num_locals,
            "Read non-existing local variable: {}.",
            local_idx);

    self.fp + FIRST_LOCAL_OFFSET + 2 * local_idx as usize
  }

  pub fn read_local(&self, local_idx: u8) -> u16 {
    // TODO: cache the num_locals in the Stack struct
    // let num_locals = self.stack[NUM_LOCALS_OFFSET as usize];
    // assert!(local_idx < num_locals, "Read non-existing local variable.");
    // let offset = FIRST_LOCAL_OFFSET + 2 * local_idx as usize;
    let offset = self.offset_for_local(local_idx);
    BigEndian::read_u16(&self.stack[offset..])
  }

  pub fn write_local(&mut self, local_idx: u8, val: u16) {
    let offset = self.offset_for_local(local_idx);
    BigEndian::write_u16(&mut self.stack[offset..], val);
  }

  fn push_u32(&mut self, val: u32) {
    let mut slice = self.stack.as_mut_slice();
    BigEndian::write_u32(&mut slice[self.sp..], val);
    self.sp += 4;
  }

  pub fn push_u16(&mut self, val: u16) {
    let mut slice = self.stack.as_mut_slice();
    BigEndian::write_u16(&mut slice[self.sp..], val);
    self.sp += 2;
  }

  pub fn pop_u16(&mut self) -> u16 {
    self.sp -= 2;
    BigEndian::read_u16(&self.stack[self.sp..])
  }

  fn push_u8(&mut self, val: u8) {
    self.stack[self.sp] = val;
    self.sp += 1;
  }
}

#[cfg(test)]
mod test {
  use byteorder::{BigEndian, ByteOrder};
  use std::u16;
  use super::{FIRST_LOCAL_OFFSET, Stack};
  use super::super::vm::VariableRef;

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
  fn test_push_pop_u16() {
    let mut stack = Stack::new(100);
    stack.push_u16(5);
    stack.push_u16(10);
    stack.push_u16(15);
    assert_eq!(&stack.stack[stack.base_sp..stack.base_sp + 6],
               vec![0, 5, 0, 10, 0, 15].as_slice());

    assert_eq!(15, stack.pop_u16());
    stack.push_u16(0xbcaa);
    assert_eq!(0xbcaa, stack.pop_u16());
    assert_eq!(10, stack.pop_u16());
    assert_eq!(5, stack.pop_u16());
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

  #[test]
  fn test_local() {
    let mut stack = Stack::new(100);
    let num_locals = 5;
    stack.new_frame(0x2345, num_locals, VariableRef::Local(3));

    for i in 0..num_locals {
      assert_eq!(0, stack.read_local(i));
    }

    stack.write_local(0, 0xbe55);

    assert_eq!(0xbe55, stack.read_local(0));

    stack.write_local(4, 0xccee);
    assert_eq!(0xccee, stack.read_local(4));
  }

  #[test]
  fn test_frames() {
    let mut stack = Stack::new(256);

    // The stack is initialized with a base stack frame with no locals.
    assert_eq!(FIRST_LOCAL_OFFSET, stack.sp);
    let old_fp = stack.fp;
    let result_location = VariableRef::Local(3);

    stack.new_frame(0x8888, 5, result_location);
    // Check that the new values are as expected.
    assert_eq!(FIRST_LOCAL_OFFSET, stack.fp);
    assert_eq!(FIRST_LOCAL_OFFSET * 2 + 5 * 2, stack.sp);
    assert_eq!(stack.sp, stack.base_sp);

    // Now check that the stack contents are correct.
    assert_eq!(old_fp,
               BigEndian::read_u16(&stack.stack[stack.fp..]) as usize);
    assert_eq!(0x8888,
               BigEndian::read_u32(&stack.stack[stack.fp + super::PC_OFFSET..]));
    assert_eq!(5, stack.stack[stack.fp + super::NUM_LOCALS_OFFSET]);
    assert_eq!(VariableRef::encode(result_location),
               stack.stack[stack.fp + super::RESULT_LOCATION_OFFSET]);

    // Check that stuff is restored after popping.
    let (popped_pc, popped_location) = stack.pop_frame();
    assert_eq!(stack.fp, old_fp);
    assert_eq!(0x8888, popped_pc);
    assert_eq!(result_location, popped_location);
  }
}
