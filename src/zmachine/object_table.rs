use super::memory::Memory;

pub struct ObjectTable {
  table_offset: usize,
}

impl ObjectTable {
  pub fn new(memory: &Memory) -> ObjectTable {
    ObjectTable { table_offset: memory.property_table_offset() }
  }

  pub fn dump(&self, memory: &Memory) {
    // println!("Property table offset: {:x}", helper.base);

    // for i in 0..100 {
    //   helper.dump_object(i)
    // }
  }

  pub fn attributes(&self, memory: &Memory, object_number: u16) -> u32 {
    let object_offset = self.table_offset + 31 * 2 + (object_number as usize - 1) * 9;
    memory.u32_at_index(object_offset)
  }

  pub fn put_property(&mut self,
                      memory: &mut Memory,
                      object_index: u16,
                      property_number: u16,
                      value: u16) {
    let object_offset = self.table_offset + 31 * 2 + (object_index as usize - 1) * 9;

    let prop_header_offset = memory.u16_at_index(object_offset + 7) as usize;

    let text_length = memory.u8_at_index(prop_header_offset);

    // 1 to skip the size byte, 2 * text length to skip the description
    let mut prop_offset = prop_header_offset + 1 + 2 * text_length as usize;

    let mut i = 0;
    loop {
      let size_byte = memory.u8_at_index(prop_offset);
      if size_byte == 0 {
        // We've run off the end of the property table.
        break;
      }
      let prop_num = size_byte % 32;
      let prop_size = size_byte / 32 + 1;
      if prop_num as u16 == property_number {
        // skip the size byte.
        let value_offset = prop_offset + 1;
        if prop_size == 2 {
          memory.set_u16_at_index(value_offset, value);
        } else if prop_size == 1 {
          memory.set_index_to_u8(value_offset, value as u8);
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
