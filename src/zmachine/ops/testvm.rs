use byteorder::{BigEndian, ByteOrder};
use result::Result;
use zmachine::vm::{BytePtr, Memory, RawPtr, VM, VariableRef, WordPtr};
use zmachine::vm::test::{MockObjectTable, MockObjectTableStorage, MockPropertyTable,
                         MockPropertyTableStorage};

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
  pub object_storage: MockObjectTableStorage,
  pub property_storage: MockPropertyTableStorage,
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
      object_storage: MockObjectTableStorage::new(),
      property_storage: MockPropertyTableStorage::new(),
    }
  }

  pub fn set_heap(&mut self, heap: Vec<u8>) {
    self.heap = heap;
  }

  pub fn set_pc(&mut self, pc: usize) {
    self.pc = pc;
  }

  pub fn set_pcbytes(&mut self, pcbytes: Vec<u8>) {
    self.pcbytes = pcbytes;
    self.pc = 0;
  }

  pub fn set_jump_offset_byte(&mut self, offset: u8, polarity: bool) {
    let mut byte = 0b01000000u8;
    if polarity {
      byte |= 0b10000000;
    }
    byte |= offset & 0b00111111;
    self.set_pcbytes(vec![byte]);
  }

  pub fn set_jump_offset_word(&mut self, offset: i16, polarity: bool) {
    let mut word = 0u16;
    if polarity {
      word |= 0b1000000000000000;
    }
    word |= (offset as u16) & 0b0011111111111111;
    let mut vec = vec![0u8, 0u8];
    BigEndian::write_u16(vec.as_mut_slice(), word as u16);
    self.set_pcbytes(vec);
  }
}

impl VM for TestVM {
  type ObjTable = MockObjectTable;
  type ObjStorage = MockObjectTableStorage;
  type PropertyTable = MockPropertyTable;
  type PropertyAccess = MockPropertyTableStorage;

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

  fn pop_frame(&mut self) -> Result<(usize, VariableRef)> {
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

  fn object_table(&self) -> Result<Self::ObjTable> {
    Ok(MockObjectTable::new())
  }

  fn object_storage(&self) -> &Self::ObjStorage {
    &self.object_storage
  }

  fn object_storage_mut(&mut self) -> &mut Self::ObjStorage {
    &mut self.object_storage
  }

  fn property_storage(&self) -> &Self::PropertyAccess {
    &self.property_storage
  }

  fn property_storage_mut(&mut self) -> &mut Self::PropertyAccess {
    &mut self.property_storage
  }

  // fn attributes(&mut self, object_number: u16) -> Result<u32> {
  //   unimplemented!()
  // }

  // fn set_attributes(&mut self, object_number: u16, attrs: u32) -> Result<()> {
  //   unimplemented!()
  // }

  // fn parent_number(&self, object_number: u16) -> Result<u16> {
  //   unimplemented!()
  // }

  // fn child_number(&self, object_number: u16) -> Result<u16> {
  //   unimplemented!()
  // }

  // fn sibling_number(&self, object_number: u16) -> Result<u16> {
  //   unimplemented!()
  // }

  // fn put_property(&mut self, object_number: u16, property_index: u16, value: u16) -> Result<()> {
  //   unimplemented!()
  // }
  // fn insert_obj(&mut self, object_number: u16, dest_number: u16) -> Result<()> {
  //   unimplemented!()
  // }
  // fn object_name(&self, object_number: u16) -> Result<RawPtr> {
  //   unimplemented!()
  // }
  // fn get_property(&self, object_number: u16, property_number: u16) -> Result<u16> {
  //   unimplemented!()
  // }

  fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> Result<WordPtr> {
    unimplemented!()
  }
}
