use zmachine::vm::{BytePtr, Memory};

// We create traits for ZObjectTable and ZObject to facilitate testability.
// In the zmachine, these are memory mapped into the VM's dynamic memory, but
// setting up memory with objects is a pain in the butt, and _verifying_ them
// and _debugging_ with them is an even bigger pain. (Especially when we start
// working with different versions of the zspec that have different layouts for
// the object table.)
//
// So, we will implement and test the memory-mapped versions, but we will use
// a struct-based implementation to test the higher-level functions.
pub trait ZObjectTable {
  type ZObject: ZObject<Helper = Self::Helper, PropertyTable = Self::PropertyTable>;
  type Helper;
  type PropertyTable: ZPropertyTable;

  fn object_with_number(&self, object_number: u16) -> Self::ZObject;
}

pub trait ZObject {
  type Helper;
  type PropertyTable;

  fn attributes(&self, helper: &Self::Helper) -> u32;
  fn set_attributes(&self, attrs: u32, helper: &mut Self::Helper);
  fn parent(&self, helper: &Self::Helper) -> u16;
  fn set_parent(&self, parent: u16, helper: &mut Self::Helper);
  fn sibling(&self, helper: &Self::Helper) -> u16;
  fn set_sibling(&self, sibling: u16, helper: &mut Self::Helper);
  fn child(&self, helper: &Self::Helper) -> u16;
  fn set_child(&self, child: u16, helper: &mut Self::Helper);
  fn property_table(&self, helper: &Self::Helper) -> Self::PropertyTable;
}

pub trait ZPropertyAccess {
  type Ref;

  fn set_byte_property(&mut self, value: u8, ptr: Self::Ref);
  fn set_word_property(&mut self, value: u16, ptr: Self::Ref);
}

impl ZPropertyAccess for Memory {
  type Ref = BytePtr;

  fn set_byte_property(&mut self, value: u8, ptr: BytePtr) {
    self.set_u8_at(value, ptr);
  }
  fn set_word_property(&mut self, value: u16, ptr: BytePtr) {
    self.set_u16_at(value, ptr);
  }
}

pub trait ZPropertyTable {
  type Helper;
  type Ref;

  fn name_ptr(&self, helper: &Self::Helper) -> Self::Ref;
  // property numbers are 1-31. Returns the size and ptr to the property.
  fn find_property(&self, number: u16, helper: &Self::Helper) -> Option<(u16, Self::Ref)>;

  fn set_property(&self, number: u16, value: u16, helper: &mut Self::Helper)
    where Self::Helper: ZPropertyAccess<Ref = Self::Ref> {
    if let Some((size, ptr)) = self.find_property(number, helper) {
      match size {
        1 => helper.set_byte_property(value as u8, ptr),
        2 => helper.set_word_property(value, ptr),
        _ => {
          panic!("Invalid size, {}, for property {}.", size, number);
        }
      }
    } else {
      // Not found, the spec says to halt. Ugh.
      panic!("Property {} not found.", number);
    }
  }
}

#[derive(Debug)]
pub struct MemoryMappedObjectTable {
  base_ptr: BytePtr,
}

impl MemoryMappedObjectTable {
  pub fn new(ptr: BytePtr) -> MemoryMappedObjectTable {
    MemoryMappedObjectTable { base_ptr: ptr }
  }
}

impl ZObjectTable for MemoryMappedObjectTable {
  type ZObject = MemoryMappedObject;
  type Helper = Memory;
  type PropertyTable = MemoryMappedPropertyTable;

  fn object_with_number(&self, object_number: u16) -> MemoryMappedObject {
    // TODO: check for 0.
    MemoryMappedObject {
      number: object_number,
      // 31 * 2 to skip the defaults table.
      // Subtract one from object_number because objects are 1-indexed.
      ptr: self.base_ptr.inc_by(31 * 2 + (object_number - 1) * 9),
    }
  }
}

#[derive(Debug)]
struct MemoryMappedObject {
  number: u16,
  ptr: BytePtr,
}

impl ZObject for MemoryMappedObject {
  type Helper = Memory;
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

#[derive(Debug)]
pub struct MemoryMappedPropertyTable {
  ptr: BytePtr,
  text_len: u8,
}

impl ZPropertyTable for MemoryMappedPropertyTable {
  type Helper = Memory;
  type Ref = BytePtr;

  fn name_ptr(&self, helper: &Memory) -> BytePtr {
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
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::{MemoryMappedObject, MemoryMappedObjectTable, MemoryMappedPropertyTable, ZObject,
              ZObjectTable, ZPropertyAccess, ZPropertyTable};
  use zmachine::vm::{BytePtr, Memory};

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

    assert_eq!(Some((2, BytePtr::new(9))),
               prop_table.find_property(12, &memory));
    assert_eq!(Some((4, BytePtr::new(12))),
               prop_table.find_property(7, &memory));
    assert_eq!(Some((2, BytePtr::new(17))),
               prop_table.find_property(3, &memory));
  }

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
  }

  // In theory, if we establish confidence in the memory-mapped versions of the
  // object table, then we can test the default functions in terms of another
  // implementation. The main benefit of this is that we don't have to set up
  // a Memory containing the data which really helps with debugging.
  pub struct MockPropertyTable {
    table: HashMap<u16, MockProperty>,
  }

  impl MockPropertyTable {
    fn new(size: u16) -> MockPropertyTable {
      MockPropertyTable { table: HashMap::new() }
    }

    fn add_property(&mut self, number: u16, size: u16, val: u16) {
      self.table.insert(number, MockProperty::new(number, size, val));
    }
  }

  #[derive(Clone, Copy)]
  struct MockProperty {
    number: u16,
    size: u16,
    val: u16,
  }

  impl MockProperty {
    fn new(number: u16, size: u16, val: u16) -> MockProperty {
      MockProperty {
        number: number,
        size: size,
        val: val,
      }
    }
  }

  impl ZPropertyTable for MockPropertyTable {
    type Helper = Self;
    type Ref = u16;

    fn name_ptr(&self, helper: &Self::Helper) -> u16 {
      0
    }

    fn find_property(&self, number: u16, helper: &Self::Helper) -> Option<(u16, Self::Ref)> {
      Some((2, number))
    }
  }

  impl ZPropertyAccess for MockPropertyTable {
    type Ref = u16;

    fn set_byte_property(&mut self, value: u8, ptr: u16) {
      let property = self.table.get_mut(&ptr).unwrap();
      property.val = value as u16;
    }

    fn set_word_property(&mut self, value: u16, ptr: u16) {
      let property = self.table.get_mut(&ptr).unwrap();
      property.val = value;
    }
  }

  // test set property
  fn test_mock_property_table() {
    let mut table = MockPropertyTable::new(20);
    table.add_property(17, 2, 0x1234);
    table.add_property(22, 5, 0);
    table.add_property(23, 1, 0x11);

    let prop = table.find_property(17, &table);
    assert!(prop.is_some());
    let prop = table.find_property(22, &table);
    assert!(prop.is_some());
    let prop = table.find_property(23, &table);
    assert!(prop.is_some());

    let prop = table.find_property(3, &table);
    assert!(prop.is_none());
  }
}
