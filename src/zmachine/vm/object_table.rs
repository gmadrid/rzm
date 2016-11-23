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
trait ZObjectTable {
  type ZObject: ZObject<Helper = Self::Helper>;
  type Helper;

  fn object_with_number(&self, object_number: u16) -> Self::ZObject;
}

trait ZObject {
  type Helper;

  fn attributes(&self, helper: &Self::Helper) -> u32;
  fn set_attributes(&self, attrs: u32, helper: &mut Self::Helper);
  fn parent(&self, helper: &Self::Helper) -> u16;
  fn set_parent(&self, parent: u16, helper: &mut Self::Helper);
  fn sibling(&self, helper: &Self::Helper) -> u16;
  fn set_sibling(&self, sibling: u16, helper: &mut Self::Helper);
  fn child(&self, helper: &Self::Helper) -> u16;
  fn set_child(&self, child: u16, helper: &mut Self::Helper);
  fn property_table(&self, helper: &Self::Helper) -> BytePtr;
}

trait ZPropertyTable {
  type Helper;

  fn name_ptr(&self, helper: &Self::Helper) -> BytePtr;
  // property numbers are 1-31. Returns the size and ptr to the property.
  fn find_property(&self, number: u16, helper: &Self::Helper) -> Option<(u16, BytePtr)>;
}

#[derive(Debug)]
struct MemoryMappedObjectTable {
  base_ptr: BytePtr,
}

impl MemoryMappedObjectTable {
  fn new(ptr: BytePtr) -> MemoryMappedObjectTable {
    MemoryMappedObjectTable { base_ptr: ptr }
  }
}

impl ZObjectTable for MemoryMappedObjectTable {
  type ZObject = MemoryMappedObject;
  type Helper = Memory;

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

  fn property_table(&self, memory: &Memory) -> BytePtr {
    BytePtr::new(memory.u16_at(self.ptr.inc_by(7)))
  }
}

#[derive(Debug)]
struct MemoryMappedPropertyTable {
  ptr: BytePtr,
  text_len: u8,
}

impl MemoryMappedPropertyTable {
  fn property_table_for_number<OT>(object_table: OT,
                                   object_number: u16,
                                   memory: &Memory)
                                   -> MemoryMappedPropertyTable
    where OT: ZObjectTable<Helper = Memory> {
    let obj = object_table.object_with_number(object_number);
    let ptr = obj.property_table(memory);
    MemoryMappedPropertyTable {
      ptr: ptr,
      text_len: memory.u8_at(ptr),
    }
  }
}

impl ZPropertyTable for MemoryMappedPropertyTable {
  type Helper = Memory;

  fn name_ptr(&self, helper: &Memory) -> BytePtr {
    self.ptr.inc_by(1)
  }

  fn find_property(&self, number: u16, memory: &Memory) -> Option<(u16, BytePtr)> {
    // * 2 because it's a word count, +1 to skip the size byte as well as the text.
    let mut prop_ptr = self.ptr.inc_by(self.text_len as u16 * 2 + 1);
    loop {
      println!("PP: {:?}", prop_ptr);
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
  use super::{MemoryMappedObject, MemoryMappedObjectTable, MemoryMappedPropertyTable, ZObject,
              ZObjectTable, ZPropertyTable};
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
    assert_eq!(BytePtr::new(0x6543), obj.property_table(&memory));

    obj.set_attributes(0x55667788, &mut memory);
    obj.set_parent(0x11, &mut memory);
    obj.set_sibling(0x77, &mut memory);
    obj.set_child(0xcc, &mut memory);

    assert_eq!(0x55667788, obj.attributes(&memory));
    assert_eq!(0x11, obj.parent(&memory));
    assert_eq!(0x77, obj.sibling(&memory));
    assert_eq!(0xcc, obj.child(&memory));
  }
}


// ===========================================================

// pub struct ObjectTable<'a> {
//   memory: &'a mut Memory,
//   base: ObjectTableBase,
// }

// // Ptr to the base of the property table (from the file header).
// #[derive(Debug)]
// struct ObjectTableBase {
//   ptr: BytePtr,
// }

// impl ObjectTableBase {
//   fn object_with_number(&self, object_number: u16) -> ObjectBase {
//     // 31 * 2 to skip the defaults table.
//     // Subtract one from object_number because objects are 1-indexed.
//     // TODO: check for 0.
//     ObjectBase {
//       number: object_number,
//       ptr: self.ptr.inc_by(31 * 2 + (object_number - 1) * 9),
//     }
//   }
// }

// // Ptr to an Object
// #[derive(Debug)]
// struct ObjectBase {
//   number: u16,
//   ptr: BytePtr,
// }

// impl ObjectBase {
//   fn attributes(&self, memory: &Memory) -> u32 {
//     memory.u32_at(self.ptr)
//   }

//   fn set_attributes(&self, attrs: u32, memory: &mut Memory) {
//     memory.set_u32_at(self.ptr, attrs);
//   }

//   fn parent(&self, memory: &Memory) -> u16 {
//     memory.u8_at(self.ptr.inc_by(4)) as u16
//   }

//   fn set_parent(&self, parent_number: u16, memory: &mut Memory) {
//     memory.set_u8_at(parent_number as u8, self.ptr.inc_by(4));
//   }

//   fn sibling(&self, memory: &Memory) -> u16 {
//     memory.u8_at(self.ptr.inc_by(5)) as u16
//   }

//   fn set_sibling(&self, sibling_number: u16, memory: &mut Memory) {
//     memory.set_u8_at(sibling_number as u8, self.ptr.inc_by(5))
//   }

//   fn child(&self, memory: &Memory) -> u16 {
//     memory.u8_at(self.ptr.inc_by(6)) as u16
//   }

//   fn set_child(&self, child_number: u16, memory: &mut Memory) {
//     memory.set_u8_at(child_number as u8, self.ptr.inc_by(6));
//   }

//   fn remove_from_parent(&self, table: &ObjectTableBase, memory: &mut Memory) {
//     let parent_number = self.parent(memory);
//     if parent_number == 0 {
//       return;
//     }

//     let parent_object = table.object_with_number(parent_number);
//     if parent_object.child(memory) == self.number {
//       // It's the first child.
//       let sibling_number = self.sibling(memory);
//       parent_object.set_child(sibling_number, memory);
//       self.set_sibling(0, memory);
//       self.set_parent(0, memory);
//     } else {
//       unimplemented!();
//     }
//   }

//   fn make_parent(&self, parent: &ObjectBase, memory: &mut Memory) {
//     let child_number = parent.child(memory);
//     parent.set_child(self.number, memory);
//     self.set_sibling(child_number, memory);
//   }
// }

// impl<'a> ObjectTable<'a> {
//   pub fn new(memory: &mut Memory) -> ObjectTable {
//     let table_offset = memory.property_table_ptr();
//     let ot = ObjectTable {
//       memory: memory,
//       base: ObjectTableBase { ptr: table_offset },
//     };
//     ot
//   }

//   pub fn attributes(&self, object_number: u16) -> u32 {
//     let object = self.base.object_with_number(object_number);
//     object.attributes(self.memory)
//   }

//   pub fn set_attributes(&mut self, object_number: u16, attrs: u32) {
//     let object = self.base.object_with_number(object_number);
//     object.set_attributes(attrs, self.memory);
//   }

//   pub fn parent(&self, object_number: u16) -> u16 {
//     let object = self.base.object_with_number(object_number);
//     object.parent(self.memory)
//   }

//   pub fn set_parent(&mut self, object_number: u16, parent: u16) {
//     let object = self.base.object_with_number(object_number);
//     object.set_parent(parent, self.memory);
//   }

//   pub fn sibling(&self, object_number: u16) -> u16 {
//     let object = self.base.object_with_number(object_number);
//     object.sibling(self.memory)
//   }

//   pub fn set_sibling(&mut self, object_number: u16, sibling: u16) {
//     let object = self.base.object_with_number(object_number);
//     object.set_sibling(sibling, self.memory);
//   }

//   pub fn child(&self, object_number: u16) -> u16 {
//     let object = self.base.object_with_number(object_number);
//     object.child(self.memory)
//   }

//   pub fn set_child(&mut self, object_number: u16, child: u16) {
//     let object = self.base.object_with_number(object_number);
//     object.set_child(child, self.memory);
//   }

//   pub fn put_property(&mut self, object_number: u16, property_index: u16, value: u16) {
//     let object = self.base.object_with_number(object_number);

//     let prop_header_ptr = BytePtr::new(self.memory.u16_at(object.ptr.inc_by(7)));
//     let text_length = self.memory.u8_at(prop_header_ptr);

//     // 1 to skip the size byte, 2 * text length to skip the description
//     let mut prop_ptr = prop_header_ptr.inc_by((1 + 2 * text_length) as u16);

//     let mut i = 0;
//     loop {
//       let size_byte = self.memory.u8_at(prop_ptr);
//       if size_byte == 0 {
//         // We've run off the end of the property table.
//         break;
//       }
//       let prop_num = size_byte % 32;
//       let prop_size = size_byte / 32 + 1;
//       if prop_num as u16 == property_index {
//         // skip the size byte.
//         let value_ptr = prop_ptr.inc_by(1);
//         if prop_size == 2 {
//           self.memory.set_u16_at(value, value_ptr);
//         } else if prop_size == 1 {
//           self.memory.set_u8_at(value as u8, value_ptr);
//         } else {
//           panic!("{:?}", "prop data size must be 0 or 1");
//         }
//         break;
//       }

//       // Spec says (size_byte / 32) + 1, plus another to skip the size byte.
//       prop_ptr = prop_ptr.inc_by((size_byte / 32 + 2) as u16);
//       i += 1;
//       if i > 10 {
//         break;
//       }
//     }
//   }

//   pub fn insert_obj(&mut self, object_number: u16, dest_number: u16) {
//     if object_number == 0 {
//       return;
//     }

//     let object = self.base.object_with_number(object_number);
//     let dest = self.base.object_with_number(dest_number);

//     object.remove_from_parent(&self.base, self.memory);
//     object.make_parent(&dest, self.memory);

//     //    self.remove_object_from_parent(object_offset);

//     // Remove from current parent.
//     // 1) if current parent is 0, then skip this step.
//     // 2) find parent, remove object.

//     // Add to new parent.
//   }
// }

// #[cfg(test)]
// mod tests_old {
//   use byteorder::{BigEndian, ByteOrder};
//   use super::ObjectTable;
//   use zmachine::vm::{BytePtr, Memory};

//   #[derive(Debug)]
//   struct ObjectDesc {
//     attributes: u32,
//     parent: u8,
//     sibling: u8,
//     child: u8,
//   }

//   impl ObjectDesc {
//     fn new(attributes: u32, parent: u8, sibling: u8, child: u8) -> ObjectDesc {
//       ObjectDesc {
//         attributes: attributes,
//         parent: parent,
//         sibling: sibling,
//         child: child,
//       }
//     }
//   }

//   fn build_test_object_table(descs: &[ObjectDesc]) -> Memory {
//     let mut bytes: Vec<u8> = Vec::new();

//     // Make room for the "header".
//     bytes.resize(0x10, 0);
//     // Then set the table index to point after the "header".
//     // Magic number: 0x0a is the property table offset.
//     let len = bytes.len();
//     BigEndian::write_u16(&mut bytes[0x0a..], len as u16);

//     let num_properties = 31;
//     let num_property_bytes = num_properties * 2;

//     let default_properties_start = bytes.len();
//     // Make room for the default properties.
//     let len = bytes.len();
//     bytes.resize(len + num_property_bytes, 0);

//     // Write some values into the default properties table so that we can
//     // distinguish them as a testing aid.
//     for i in 0..num_properties {
//       let offset = default_properties_start + 2 * i;
//       let val = (i + 1) * 3;
//       BigEndian::write_u16(&mut bytes[offset..], val as u16);
//     }

//     for desc in descs {
//       let len = bytes.len();
//       bytes.resize(len + 9, 0);

//       BigEndian::write_u32(&mut bytes[len..], desc.attributes);
//       bytes[len + 4] = desc.parent;
//       bytes[len + 5] = desc.sibling;
//       bytes[len + 6] = desc.child;
//     }

//     Memory::from(bytes)
//   }

//   #[test]
//   fn test_basics() {
//     let mut memory = build_test_object_table(vec![
//       ObjectDesc::new(17u32, 1u8, 2u8, 3u8 ),
//       ObjectDesc::new(35u32, 4u8, 5u8, 6u8 ),
//       ObjectDesc::new(17u32, 7u8, 8u8, 9u8 ),
//       ObjectDesc::new(0xfedcba98u32, 10u8, 11u8, 12u8 ),
//     ]
//       .as_slice());

//     {
//       let mut object_table = ObjectTable::new(&mut memory);

//       assert_eq!(17, object_table.attributes(1));
//       assert_eq!(35, object_table.attributes(2));
//       assert_eq!(17, object_table.attributes(3));
//       assert_eq!(0xfedcba98u32, object_table.attributes(4));

//       assert_eq!(1, object_table.parent(1));
//       assert_eq!(4, object_table.parent(2));
//       assert_eq!(7, object_table.parent(3));
//       assert_eq!(10, object_table.parent(4));

//       assert_eq!(2, object_table.sibling(1));
//       assert_eq!(5, object_table.sibling(2));
//       assert_eq!(8, object_table.sibling(3));
//       assert_eq!(11, object_table.sibling(4));

//       assert_eq!(3, object_table.child(1));
//       assert_eq!(6, object_table.child(2));
//       assert_eq!(9, object_table.child(3));
//       assert_eq!(12, object_table.child(4));

//       object_table.set_attributes(1, 0x54453443);
//       assert_eq!(0x54453443, object_table.attributes(1));
//       object_table.set_parent(2, 3);
//       assert_eq!(3, object_table.parent(2));
//       object_table.set_sibling(1, 3);
//       assert_eq!(3, object_table.sibling(1));
//       object_table.set_child(3, 1);
//       assert_eq!(1, object_table.child(3));
//     }

//     let match_memory = build_test_object_table(vec![
//       ObjectDesc::new(0x54453443, 1u8, 3u8, 3u8),
//       ObjectDesc::new(35u32, 3u8, 5u8, 6u8),
//       ObjectDesc::new(17u32, 7u8, 8u8, 1u8 ),
//       ObjectDesc::new(0xfedcba98u32, 10u8, 11u8, 12u8 ),
//       ]
//       .as_slice());
//     assert!(match_memory == memory);
//   }
// }
