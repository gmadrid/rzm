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
  type ZObject;

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
}

#[cfg(test)]
mod tests {
  use super::{MemoryMappedObjectTable, ZObjectTable};
  use zmachine::vm::BytePtr;

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
