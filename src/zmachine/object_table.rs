use super::vm::Memory;

pub struct ObjectTable<'a> {
  memory: &'a mut Memory,
  base: ObjectTableBase,
}

// Byteaddress of the base of the property table (from the file header).
struct ObjectTableBase {
  offset: usize,
}

impl ObjectTableBase {
  fn object_with_number(&self, object_number: u16) -> ObjectBase {
    // 31 * 2 to skip the defaults table.
    // Subtract one from object_number because objects are 1-indexed.
    ObjectBase {
      number: object_number,
      offset: self.offset + 31 * 2 + (object_number as usize - 1) * 9,
    }
  }
}

// Byteaddress for the Object
struct ObjectBase {
  number: u16,
  offset: usize,
}

impl ObjectBase {
  fn attributes(&self, memory: &Memory) -> u32 {
    memory.u32_at_index(self.offset)
  }

  fn parent(&self, memory: &Memory) -> u16 {
    memory.u8_at(self.offset + 4) as u16
  }

  fn set_parent(&self, parent_number: u16, memory: &mut Memory) {
    memory.set_index_to_u8(self.offset + 4, parent_number as u8);
  }

  fn sibling(&self, memory: &Memory) -> u16 {
    memory.u8_at_index(self.offset + 5) as u16
  }

  fn set_sibling(&self, sibling_number: u16, memory: &mut Memory) {
    memory.set_index_to_u8(self.offset + 5, sibling_number as u8);
  }

  fn child(&self, memory: &Memory) -> u16 {
    memory.u8_at_index(self.offset + 6) as u16
  }

  fn set_child(&self, child_number: u16, memory: &mut Memory) {
    memory.set_index_to_u8(self.offset + 6, child_number as u8);
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
    let table_offset = memory.property_table_offset();
    ObjectTable {
      memory: memory,
      base: ObjectTableBase { offset: table_offset },
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

    let prop_header_offset = self.memory.u16_at_index(object.offset + 7) as usize;

    let text_length = self.memory.u8_at_index(prop_header_offset);

    // 1 to skip the size byte, 2 * text length to skip the description
    let mut prop_offset = prop_header_offset + 1 + 2 * text_length as usize;

    let mut i = 0;
    loop {
      let size_byte = self.memory.u8_at_index(prop_offset);
      if size_byte == 0 {
        // We've run off the end of the property table.
        break;
      }
      let prop_num = size_byte % 32;
      let prop_size = size_byte / 32 + 1;
      if prop_num as u16 == property_index {
        // skip the size byte.
        let value_offset = prop_offset + 1;
        if prop_size == 2 {
          self.memory.set_u16_at_index(value_offset, value);
        } else if prop_size == 1 {
          self.memory.set_index_to_u8(value_offset, value as u8);
        } else {
          panic!("{:?}", "prop data size must be 0 or 1");
        }
        break;
      }

      // Spec says (size_byte / 32) + 1, plus another to skip the size byte.
      prop_offset += ((size_byte / 32) + 2) as usize;
      i += 1;
      if i > 10 {
        break;
      }
    }
  }
}
