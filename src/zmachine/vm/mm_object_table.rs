use zmachine::vm::memory::Memory;
use zmachine::vm::object_table::{ZObject, ZObjectTable, ZPropertyAccess, ZPropertyTable};
use zmachine::vm::ptrs::{BytePtr, RawPtr};

#[derive(Debug)]
pub struct MemoryMappedObjectTable {
  base_ptr: BytePtr,
}

impl MemoryMappedObjectTable {
  pub fn new(ptr: BytePtr) -> MemoryMappedObjectTable {
    MemoryMappedObjectTable { base_ptr: ptr }
  }
}

#[derive(Debug)]
pub struct MemoryMappedObject {
  number: u16,
  ptr: BytePtr,
}

#[derive(Debug)]
pub struct MemoryMappedPropertyTable {
  ptr: BytePtr,
  text_len: u8,
}

impl ZObjectTable for MemoryMappedObjectTable {
  type ZObject = MemoryMappedObject;
  type DataAccess = Memory;
  type PropertyTable = MemoryMappedPropertyTable;
  type PropertyAccess = Memory;

  fn object_with_number(&self, object_number: u16) -> MemoryMappedObject {
    // TODO: check for 0.
    MemoryMappedObject {
      number: object_number,
      // 31 * 2 to skip the defaults table.
      // Subtract one from object_number because objects are 1-indexed.
      ptr: self.base_ptr.inc_by(31 * 2 + (object_number - 1) * 9),
    }
  }

  fn default_property_value(&self, property_number: u16, access: &Memory) -> u16 {
    // TODO: test this.
    let ptr = self.base_ptr.inc_by(2 * (property_number - 1));
    access.u16_at(ptr)
  }
}

impl ZObject for MemoryMappedObject {
  type DataAccess = Memory;
  type PropertyTable = MemoryMappedPropertyTable;

  fn attributes(&self, memory: &Memory) -> u32 {
    memory.u32_at(self.ptr)
  }

  fn set_attributes(&self, attrs: u32, memory: &mut Memory) {
    memory.set_u32_at(self.ptr, attrs);
  }

  fn parent(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.ptr.inc_by(4)) as u16
  }

  fn set_parent(&self, parent: u16, memory: &mut Memory) {
    memory.set_u8_at(parent as u8, self.ptr.inc_by(4));
  }

  fn sibling(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.ptr.inc_by(5)) as u16
  }

  fn set_sibling(&self, sibling: u16, memory: &mut Memory) {
    memory.set_u8_at(sibling as u8, self.ptr.inc_by(5))
  }

  fn child(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.ptr.inc_by(6)) as u16
  }

  fn set_child(&self, child: u16, memory: &mut Memory) {
    memory.set_u8_at(child as u8, self.ptr.inc_by(6));
  }

  fn property_table(&self, memory: &Memory) -> MemoryMappedPropertyTable {
    let ptr = BytePtr::new(memory.u16_at(self.ptr.inc_by(7)));
    MemoryMappedPropertyTable {
      ptr: ptr,
      text_len: memory.u8_at(ptr),
    }
  }
}

impl ZPropertyTable for MemoryMappedPropertyTable {
  type PropertyAccess = Memory;

  fn name_ptr(&self, _: &Memory) -> BytePtr {
    self.ptr.inc_by(1)
  }

  fn find_property(&self, number: u16, memory: &Memory) -> Option<(u16, BytePtr)> {
    // * 2 because it's a word count, +1 to skip the size byte as well as the text.
    let mut prop_ptr = self.ptr.inc_by(self.text_len as u16 * 2 + 1);
    loop {
      let size_byte = memory.u8_at(prop_ptr);
      let prop_num = (size_byte & 0b00011111u8) as u16;
      // Properties are sorted descending, and terminated by a 0 size_byte.
      if prop_num < number {
        return None;
      }

      let size = size_byte / 32 + 1;
      if prop_num == number {
        return Some((size as u16, prop_ptr.inc_by(1)));
      }

      // Add 1 (for the size byte) plus the length of the property.
      prop_ptr = prop_ptr.inc_by(size as u16 + 1);
    }
  }

  fn next_property(&self, number: u16, memory: &Memory) -> u16 {
    // * 2 because text_len is a word count, +1 to skip the size byte as well as the text.
    let mut prop_ptr = self.ptr.inc_by(self.text_len as u16 * 2 + 1);
    info!(target: "pctrace", "start ptr: {:?}", prop_ptr);
    // TODO: oh, man, seriously test this.
    if number > 0 {
      match self.find_property(number, memory) {
        None => panic!("Unknown property requested: {}", number),
        Some((_, ptr)) => {
          // Subtract one to get back to the size byte.
          let val = usize::from(RawPtr::from(ptr)) - 1usize;
          prop_ptr = BytePtr::new(val as u16);
          let size_byte = memory.u8_at(prop_ptr);
          let size = size_byte / 32 + 1;
          prop_ptr = prop_ptr.inc_by(size as u16 + 1);
        }
      }
    }

    // Now, prop_ptr should point to the next property.
    info!(target: "pctrace", "end ptr: {:?}", prop_ptr);
    let size_byte = memory.u8_at(prop_ptr);
    let prop_num = (size_byte & 0b00011111u8) as u16;
    prop_num

  }
}

impl ZPropertyAccess for Memory {
  // TODO: test ZPropertyAccess for Memory
  fn byte_property(&self, ptr: BytePtr) -> u16 {
    self.u8_at(ptr) as u16
  }

  fn word_property(&self, ptr: BytePtr) -> u16 {
    self.u16_at(ptr)
  }

  fn set_byte_property(&mut self, value: u8, ptr: BytePtr) {
    self.set_u8_at(value, ptr);
  }
  fn set_word_property(&mut self, value: u16, ptr: BytePtr) {
    self.set_u16_at(value, ptr);
  }
}

#[cfg(test)]
mod tests {
  use super::{MemoryMappedObject, MemoryMappedObjectTable, MemoryMappedPropertyTable};
  use zmachine::vm::memory::Memory;
  use zmachine::vm::object_table::{ZObject, ZObjectTable, ZPropertyTable};
  use zmachine::vm::ptrs::BytePtr;

  #[test]
  fn test_mm_object_table() {
    // This is a very simple data structure. We only really have to test that
    // objects are mapped to the correct place.
    let object_table = MemoryMappedObjectTable::new(BytePtr::new(0));
    let object = object_table.object_with_number(1);
    assert_eq!(1, object.number);
    assert_eq!(BytePtr::new(62), object.ptr);

    let object = object_table.object_with_number(2);
    assert_eq!(2, object.number);
    assert_eq!(BytePtr::new(71), object.ptr);

    let object = object_table.object_with_number(3);
    assert_eq!(3, object.number);
    assert_eq!(BytePtr::new(80), object.ptr);

    let object = object_table.object_with_number(6);
    assert_eq!(6, object.number);
    assert_eq!(BytePtr::new(107), object.ptr);
  }

  #[test]
  fn test_mm_objects() {
    // Again, we have kept this very simple. We only need to test that all
    // fields can be read and written.
    // But now, we have to create a Memory object to map to.
    // This requires knowledge of the spec.
    let mut memory = Memory::from(vec![0x00, 0x00, 0x00, 0x00 /* some padding */, 0x34,
                                       0x56, 0x78, 0x9a /* attributes */,
                                       0x12 /* parent */, 0x13 /* sibling */,
                                       0x23 /* child */, 0x65, 0x43 /* property ptr */]);
    let ptr = BytePtr::new(0x04);  // skip the padding
    let obj = MemoryMappedObject {
      number: 3,
      ptr: ptr,
    };

    assert_eq!(0x3456789a, obj.attributes(&memory));
    assert_eq!(0x12, obj.parent(&memory));
    assert_eq!(0x13, obj.sibling(&memory));
    assert_eq!(0x23, obj.child(&memory));

    obj.set_attributes(0x55667788, &mut memory);
    obj.set_parent(0x11, &mut memory);
    obj.set_sibling(0x77, &mut memory);
    obj.set_child(0xcc, &mut memory);

    assert_eq!(0x55667788, obj.attributes(&memory));
    assert_eq!(0x11, obj.parent(&memory));
    assert_eq!(0x77, obj.sibling(&memory));
    assert_eq!(0xcc, obj.child(&memory));

    // TODO: test property_table().
  }

  #[test]
  fn test_mm_property_table() {
    let blob = vec![// Leave 3 zeros at the front to test the ptr code.
                    0x00,
                    0x00,
                    0x00,

                    // text length of "quux" = 2 words
                    0x02,

                    // this is the encoding for "quux"
                    0x5b,
                    0x5a,
                    0xf4,
                    0xa5,

                    // property 20 of length 8
                    (20 + (8 - 1) * 32u8),
                    0x01,
                    0x02,
                    0x03,
                    0x04,
                    0x05,
                    0x06,
                    0x07,
                    0x08,

                    // property 12 of length 2, val = 0xdde2
                    (12 + (2 - 1) * 32u8),
                    0xdd,
                    0xe2,

                    // property 7 of length 4, val = 0xaacb3211
                    (7 + (4 - 1) * 32u8),
                    0xaa,
                    0xcb,
                    0x32,
                    0x11,

                    // property 3 of length 2, val = 0x1234
                    (3 + (2 - 1) * 32u8),
                    0x12,
                    0x34,

                    // termination byte
                    0x00];
    let memory = Memory::from(blob);
    let ptr = BytePtr::new(3);
    let prop_table = MemoryMappedPropertyTable {
      ptr: ptr,
      text_len: memory.u8_at(ptr),
    };

    assert_eq!(None, prop_table.find_property(21, &memory));
    assert_eq!(None, prop_table.find_property(9, &memory));
    assert_eq!(None, prop_table.find_property(1, &memory));

    assert_eq!(Some((2, BytePtr::new(18))),
               prop_table.find_property(12, &memory));
    assert_eq!(Some((4, BytePtr::new(21))),
               prop_table.find_property(7, &memory));
    assert_eq!(Some((2, BytePtr::new(26))),
               prop_table.find_property(3, &memory));
  }
}
