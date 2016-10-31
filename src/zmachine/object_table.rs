use super::memory::Memory;

pub struct ObjectTable {}
pub struct ObjectTableHelper<'a> {
  memory: &'a Memory,
  base: usize,
}
pub struct ObjectHelper<'a> {
  memory: &'a Memory,
  base: usize,
}

impl ObjectTable {
  pub fn new() -> ObjectTable {
    ObjectTable {}
  }

  pub fn dump(&self, memory: &Memory) {
    let helper = ObjectTableHelper::new(memory);
    println!("Property table offset: {:x}", helper.base);

    for i in 0..100 {
      helper.dump_object(i)
    }
  }
}

impl<'a> ObjectTableHelper<'a> {
  fn new(memory: &'a Memory) -> ObjectTableHelper {
    ObjectTableHelper {
      memory: memory,
      base: memory.property_table_offset(),
    }
  }

  fn dump_object(&self, object_number: u16) {}

  fn object_base(&self, object_number: u16) -> usize {
    self.base + object_number as usize * 9
  }
}
