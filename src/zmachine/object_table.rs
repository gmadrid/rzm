use super::vm::Memory;
use zmachine::vm::BytePtr;

pub struct ObjectTable<'a> {
  memory: &'a mut Memory,
  base: ObjectTableBase,
}

// Ptr to the base of the property table (from the file header).
struct ObjectTableBase {
  ptr: BytePtr,
}

impl ObjectTableBase {
  fn object_with_number(&self, object_number: u16) -> ObjectBase {
    // 31 * 2 to skip the defaults table.
    // Subtract one from object_number because objects are 1-indexed.
    ObjectBase {
      number: object_number,
      ptr: self.ptr.inc_by(31 * 2 + (object_number - 1) * 9),
    }
  }
}

// Ptr to an Object
struct ObjectBase {
  number: u16,
  ptr: BytePtr,
}

impl ObjectBase {
  fn attributes(&self, memory: &Memory) -> u32 {
    memory.u32_at(self.ptr)
  }

  fn parent(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.ptr.inc_by(4)) as u16
  }

  fn set_parent(&self, parent_number: u16, memory: &mut Memory) {
    memory.set_u8_at(parent_number as u8, self.ptr.inc_by(4));
  }

  fn sibling(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.ptr.inc_by(5)) as u16
  }

  fn set_sibling(&self, sibling_number: u16, memory: &mut Memory) {
    memory.set_u8_at(sibling_number as u8, self.ptr.inc_by(5))
  }

  fn child(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.ptr.inc_by(6)) as u16
  }

  fn set_child(&self, child_number: u16, memory: &mut Memory) {
    memory.set_u8_at(child_number as u8, self.ptr.inc_by(6));
  }

  fn remove_from_parent(&self, table: &ObjectTableBase, memory: &mut Memory) {
    let parent_number = self.parent(memory);
    if parent_number == 0 {
      return;
    }

    let parent_object = table.object_with_number(parent_number);
    if parent_object.child(memory) == self.number {
      // It's the first child.
      let sibling_number = self.sibling(memory);
      parent_object.set_child(sibling_number, memory);
      self.set_sibling(0, memory);
      self.set_parent(0, memory);
    } else {
      unimplemented!();
    }
  }

  fn make_parent(&self, parent: &ObjectBase, memory: &mut Memory) {
    let child_number = parent.child(memory);
    parent.set_child(self.number, memory);
    self.set_sibling(child_number, memory);
  }
}

impl<'a> ObjectTable<'a> {
  pub fn new(memory: &mut Memory) -> ObjectTable {
    let table_offset = memory.property_table_ptr();
    ObjectTable {
      memory: memory,
      base: ObjectTableBase { ptr: table_offset },
    }
  }

  pub fn insert_obj(&mut self, object_number: u16, dest_number: u16) {
    if object_number == 0 {
      return;
    }

    let object = self.base.object_with_number(object_number);
    let dest = self.base.object_with_number(dest_number);

    object.remove_from_parent(&self.base, self.memory);
    object.make_parent(&dest, self.memory);

    //    self.remove_object_from_parent(object_offset);

    // Remove from current parent.
    // 1) if current parent is 0, then skip this step.
    // 2) find parent, remove object.

    // Add to new parent.
  }

  pub fn attributes(&self, object_number: u16) -> u32 {
    let object = self.base.object_with_number(object_number);
    object.attributes(self.memory)
  }

  pub fn put_property(&mut self, object_number: u16, property_index: u16, value: u16) {
    let object = self.base.object_with_number(object_number);

    let prop_header_ptr = BytePtr::new(self.memory.u16_at(object.ptr.inc_by(7)));
    let text_length = self.memory.u8_at(prop_header_ptr);

    // 1 to skip the size byte, 2 * text length to skip the description
    let mut prop_ptr = prop_header_ptr.inc_by((1 + 2 * text_length) as u16);

    let mut i = 0;
    loop {
      let size_byte = self.memory.u8_at(prop_ptr);
      if size_byte == 0 {
        // We've run off the end of the property table.
        break;
      }
      let prop_num = size_byte % 32;
      let prop_size = size_byte / 32 + 1;
      if prop_num as u16 == property_index {
        // skip the size byte.
        let value_ptr = prop_ptr.inc_by(1);
        if prop_size == 2 {
          self.memory.set_u16_at(value, value_ptr);
        } else if prop_size == 1 {
          self.memory.set_u8_at(value as u8, value_ptr);
        } else {
          panic!("{:?}", "prop data size must be 0 or 1");
        }
        break;
      }

      // Spec says (size_byte / 32) + 1, plus another to skip the size byte.
      prop_ptr = prop_ptr.inc_by((size_byte / 32 + 2) as u16);
      i += 1;
      if i > 10 {
        break;
      }
    }
  }
}
