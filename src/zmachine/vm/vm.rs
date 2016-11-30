// YES

use result::Result;
use super::ptrs::{RawPtr, WordPtr};

/// Trait for an abstract mid-level virtual machine for running the ZMachine.
///
/// This trait treats ZMachine constructs (memory, stack, variables, pointers,
/// etc.) as one level above raw bytes. ZMachine opcodes are written in term of
/// the vm::VM trait, allowing simple testing of opcodes without setting up an
/// entire ZMachine memory dump.
///
/// * PC - the "program counter" for the running machine. Always points to the
///   next opcode/operand to be read.
/// * Memory - Controlled access to the contents of the zfile. All of the other
///   ZMachine constructs use this to read/write raw memory from the file.
/// * Stack - A stack for local variables as well as Z-functions to use as temp
///   scratch space. Also serves as a call stack.
/// * Pointers - Represent offsets into the Memory structure. The ZMachine has
///   three different treatments for offsets into Memory, each of which may
///   behave differently on each version of the ZMachine. This attempts to
///   encapsulate that complexity.
///
pub trait VM: Sized {
  /// Advance the PC past the next byte, returning that byte.
  fn read_pc_byte(&mut self) -> u8;
  /// Advance the PC past the next word, returning that word.
  fn read_pc_word(&mut self) -> u16;
  /// Return the current value of the PC.
  fn current_pc(&self) -> usize;  // TODO: make this take a RawPtr.
  /// Set the PC to the supplied value.
  fn set_current_pc(&mut self, pc: usize) -> Result<()>;  // TODO: make RawPtr
  /// Add the `offset`, treated as a 14-bit signed int, to the PC.
  fn offset_pc(&mut self, offset: i16) -> Result<()>;

  /// Create a new stack frame.
  /// * `ret_pc` - the PC value to return when the frame is popped.
  /// * `num_locals` - the number of locals (<= 8) to allocate in the new frame.
  /// * `result_location` - the VariableRef to return when the frame is popped.
  fn new_frame(&mut self,
               ret_pc: usize,
               num_locals: u8,
               result_location: VariableRef)
               -> Result<()>;

  /// Pop the current frame, the stack to its state before the frame was created.
  /// Returns the `ret_pc` and `result_location` values that were passed to the
  /// matching `new_frame` call.
  fn pop_frame(&mut self) -> Result<(usize, VariableRef)>;

  /// Pop a word value off the stack, returning that word.
  /// NOTE: prefer read_variable().
  fn pop_stack(&mut self) -> Result<u16>;
  /// Push `val` onto the stack.
  /// NOTE: prefer write_variable().
  fn push_stack(&mut self, val: u16) -> Result<()>;

  /// Read the local variable at `local_idx` from the current stack frame.
  /// `local_idx` is zero-indexed and must be less than the number of locals
  /// in the current stack frame.
  /// NOTE: prefer read_variable()
  fn read_local(&self, local_idx: u8) -> Result<u16>;
  /// Write `val` into the local variable at `local_idx` in the current stack frame.
  /// `local_idx` is zero-indexed and must be less than the number of locals
  /// in the current stack frame.
  /// NOTE: prefer write_variable().
  fn write_local(&mut self, local_idx: u8, val: u16) -> Result<()>;

  /// Read the global variable at `global_idx`.
  /// `global_idx` is in the range [0, 239].
  /// NOTE: prefer read_variable()
  fn read_global(&self, global_idx: u8) -> Result<u16>;
  /// Write `val` into the global va at `global_idx`.
  /// `global_idx` is in the range [0, 239].
  /// NOTE: prefer write_variable().
  fn write_global(&mut self, global_idx: u8, val: u16) -> Result<()>;

  /// Read the word at `ptr` in the vm's memory.
  fn read_memory<T>(&self, ptr: T) -> Result<u16> where T: Into<RawPtr>;
  /// Write `val` at `ptr` in the vm's memory.
  fn write_memory<T>(&mut self, ptr: T, val: u16) -> Result<()> where T: Into<RawPtr>;
  /// Read the single byte at `ptr` in the vm's memory.
  fn read_memory_u8<T>(&self, ptr: T) -> Result<u8> where T: Into<RawPtr>;

  /// Return all of the attributes for the object with number `object_number`.
  fn attributes(&mut self, object_number: u16) -> Result<u32>;
  fn set_attributes(&mut self, object_number: u16, attrs: u32) -> Result<()>;
  /// Set `property_index` in `object_number` to `value`.
  fn put_property(&mut self, object_number: u16, property_index: u16, value: u16) -> Result<()>;
  /// Make the object at `object_number` be the first child of the object
  /// at `dest_number`.
  fn insert_obj(&mut self, object_number: u16, dest_number: u16) -> Result<()>;

  /// Return the address as a WordPtr of the specified abbrev.
  fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> Result<WordPtr>;

  /// Read the value from the specified variable.
  fn read_variable(&mut self, variable: VariableRef) -> Result<u16> {
    match variable {
      VariableRef::Stack => self.pop_stack(),
      VariableRef::Local(idx) => self.read_local(idx),
      VariableRef::Global(idx) => self.read_global(idx),
    }
  }

  /// Write `value` to the specified `variable`.
  fn write_variable(&mut self, variable: VariableRef, value: u16) -> Result<()> {
    match variable {
      VariableRef::Stack => self.push_stack(value),
      VariableRef::Local(idx) => self.write_local(idx, value),
      VariableRef::Global(idx) => self.write_global(idx, value),
    }
  }

  /// Convenience call for returning a value from a function call.
  /// Pops the last frame, stores `value` into the result var from the
  /// previous frame, and resets the pc from the value in the previous frame.
  fn ret_value(&mut self, value: u16) -> Result<()> {
    let (pc, result_var) = self.pop_frame()?;
    self.write_variable(result_var, value)?;
    self.set_current_pc(pc)?;
    Ok(())
  }
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum VariableRef {
  Stack,
  Local(u8),
  Global(u8),
}

impl VariableRef {
  pub fn decode(encoded: u8) -> VariableRef {
    match encoded {
      0x00 => VariableRef::Stack,
      0x01...0x0f => VariableRef::Local(encoded - 0x01),
      0x10...0xff => VariableRef::Global(encoded - 0x10),
      _ => panic!("What is this number: {}", encoded),
    }
  }

  pub fn encode(variable: VariableRef) -> u8 {
    match variable {
      VariableRef::Stack => 0,
      VariableRef::Local(local_idx) => 0x01 + local_idx,
      VariableRef::Global(global_idx) => 0x10 + global_idx,
    }
  }
}

#[cfg(test)]
mod test {
  use super::VariableRef;

  #[test]
  fn test_encode() {
    assert_eq!(0x00, VariableRef::encode(VariableRef::Stack));
    assert_eq!(0x01, VariableRef::encode(VariableRef::Local(0x00)));
    assert_eq!(0x0f, VariableRef::encode(VariableRef::Local(0x0e)));
    assert_eq!(0x10, VariableRef::encode(VariableRef::Global(0x00)));
    assert_eq!(0xff, VariableRef::encode(VariableRef::Global(0xef)));
  }

  #[test]
  fn test_decode() {
    assert_eq!(VariableRef::Stack, VariableRef::decode(0x00));
    assert_eq!(VariableRef::Local(0x00), VariableRef::decode(0x01));
    assert_eq!(VariableRef::Local(0x0e), VariableRef::decode(0x0f));
    assert_eq!(VariableRef::Global(0x00), VariableRef::decode(0x10));
    assert_eq!(VariableRef::Global(0xef), VariableRef::decode(0xff));
  }
}
