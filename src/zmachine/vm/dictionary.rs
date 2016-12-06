use zmachine::vm::{BytePtr, Memory};

#[derive(Debug)]
pub struct Dictionary {
  separators: Vec<char>,
  entry_length: u8,
  num_entries: u16,
  entries_ptr: BytePtr,
}

impl Dictionary {
  pub fn new(memory: &Memory) -> Dictionary {
    let mut ptr = memory.dictionary_table_ptr();
    let num_separators = memory.u8_at(ptr);

    let separators = Vec::<char>::new();
    ptr = ptr.inc_by(1u16 + num_separators as u16);

    let entry_length = memory.u8_at(ptr);
    ptr = ptr.inc_by(1);
    let num_entries = memory.u16_at(ptr);
    ptr = ptr.inc_by(2);

    Dictionary {
      separators: separators,
      entry_length: entry_length,
      num_entries: num_entries,
      entries_ptr: ptr,
    }
  }

  pub fn num_entries(&self) -> u16 {
    self.num_entries
  }

  pub fn entry_ptr(&self, i: u16) -> BytePtr {
    self.entries_ptr.inc_by(self.entry_length as u16 * (i - 1))
  }
}
