use byteorder::{BigEndian, ByteOrder};
use result::Result;
use super::super::vm::{BytePtr, RawPtr, VM, VariableRef, WordPtr};

/// All opcodes are implemented in terms of the VM trait.
/// This means that we can test all of the opcodes without creating an
/// entire ZMachine, as long as we implement enough functionality for the
/// opcode to execute.
pub struct TestVM {
  pub heap: Vec<u8>,
  pub stack: Vec<u16>,
  pub locals: [u16; 15],
  pub globals: [u16; 240],
  pub pc: usize,
  pub pcbytes: Vec<u8>,
}

impl TestVM {
  pub fn new() -> TestVM {
    TestVM {
      heap: vec![0; 1000],
      stack: Vec::new(),
      locals: [0; 15],
      globals: [0; 240],
      pc: 0,
      pcbytes: Vec::new(),
    }
  }
}

impl VM for TestVM {
  fn read_pc_byte(&mut self) -> u8 {
    let val = self.pcbytes[self.pc];
    self.pc += 1;
    val
  }

  fn read_pc_word(&mut self) -> u16 {
    let val = BigEndian::read_u16(&self.pcbytes[self.pc..]);
    self.pc += 2;
    val
  }

  fn current_pc(&self) -> usize {
    self.pc
  }

  fn set_current_pc(&mut self, pc: usize) -> Result<()> {
    self.pc = pc;
    Ok(())
  }

  fn offset_pc(&mut self, offset: i16) -> Result<()> {
    Ok(self.pc = ((self.pc as i32) + (offset as i32)) as usize)
  }


  fn new_frame(&mut self,
               ret_pc: usize,
               num_locals: u8,
               result_location: VariableRef)
               -> Result<()> {
    unimplemented!()
  }

  fn pop_frame(&mut self, return_val: u16) -> Result<(usize, VariableRef)> {
    unimplemented!()
  }

  fn pop_stack(&mut self) -> Result<u16> {
    Ok(self.stack.pop().unwrap())
  }

  fn push_stack(&mut self, val: u16) -> Result<()> {
    self.stack.push(val);
    Ok(())
  }

  fn read_local(&self, local_idx: u8) -> Result<u16> {
    Ok(self.locals[local_idx as usize])
  }

  fn write_local(&mut self, local_idx: u8, val: u16) -> Result<()> {
    self.locals[local_idx as usize] = val;
    Ok(())
  }

  fn read_global(&self, global_idx: u8) -> Result<u16> {
    Ok(self.globals[global_idx as usize])
  }

  fn write_global(&mut self, global_idx: u8, val: u16) -> Result<()> {
    self.globals[global_idx as usize] = val;
    Ok(())
  }

  fn read_memory<T>(&self, ptr: T) -> Result<u16>
    where T: Into<RawPtr> {
    Ok(BigEndian::read_u16(&self.heap[ptr.into().ptr()..]))
  }

  fn write_memory<T>(&mut self, ptr: T, val: u16) -> Result<()>
    where T: Into<RawPtr> {
    BigEndian::write_u16(&mut self.heap[ptr.into().ptr()..], val);
    Ok(())
  }

  fn read_memory_u8<T>(&self, ptr: T) -> Result<u8>
    where T: Into<RawPtr> {
    Ok(self.heap[ptr.into().ptr()])
  }

  fn attributes(&mut self, object_number: u16) -> Result<u32> {
    unimplemented!()
  }

  fn put_property(&mut self, object_number: u16, property_index: u16, value: u16) -> Result<()> {
    unimplemented!()
  }
  fn insert_obj(&mut self, object_number: u16, dest_number: u16) -> Result<()> {
    unimplemented!()
  }

  fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> Result<WordPtr> {
    unimplemented!()
  }
}
