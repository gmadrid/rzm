use std::collections::HashMap;

use zmachine::vm::object_table::{ZObject, ZObjectTable, ZPropertyTable};

#[derive(Debug,Clone,Eq,PartialEq)]
pub struct MockObjectTableStorage {
  objects: Vec<MockObjectRep>,
}

#[derive(Debug)]
pub struct MockObjectTable {
}

struct MockObject {
  object_number: u16,
}

#[derive(Debug,Clone,Eq,PartialEq)]
struct MockObjectRep {
  attributes: u32,
  parent: u16,
  sibling: u16,
  child: u16,
}

struct MockPropertyTable {
  table: HashMap<u16, MockProperty>,
}

#[derive(Clone, Copy)]
struct MockProperty {
  number: u16,
  size: u16,
  val: u16,
}

impl MockObjectTableStorage {
  pub fn new() -> MockObjectTableStorage {
    MockObjectTableStorage { objects: Vec::new() }
  }
  pub fn add_mock_object(&mut self, attributes: u32, parent: u16, sibling: u16, child: u16) {
    let obj = MockObjectRep {
      attributes: attributes,
      parent: parent,
      sibling: sibling,
      child: child,
    };
    self.objects.push(obj);
  }

  fn attributes(&self, object_number: u16) -> u32 {
    let ref rep = self.objects[object_number as usize - 1];
    rep.attributes
  }
  fn set_attributes(&mut self, object_number: u16, attrs: u32) {
    let ref mut rep = self.objects[object_number as usize - 1];
    rep.attributes = attrs;
  }
  fn parent(&self, object_number: u16) -> u16 {
    let ref rep = self.objects[object_number as usize - 1];
    rep.parent
  }
  fn set_parent(&mut self, object_number: u16, parent: u16) {
    let ref mut rep = self.objects[object_number as usize - 1];
    rep.parent = parent;
  }
  fn sibling(&self, object_number: u16) -> u16 {
    let ref rep = self.objects[object_number as usize - 1];
    rep.sibling
  }
  fn set_sibling(&mut self, object_number: u16, sibling: u16) {
    let ref mut rep = self.objects[object_number as usize - 1];
    rep.sibling = sibling;
  }
  fn child(&self, object_number: u16) -> u16 {
    let ref rep = self.objects[object_number as usize - 1];
    rep.child
  }
  fn set_child(&mut self, object_number: u16, child: u16) {
    let ref mut rep = self.objects[object_number as usize - 1];
    rep.child = child;
  }
}

impl MockObjectTable {
  pub fn new(storage: &MockObjectTableStorage) -> MockObjectTable {
    MockObjectTable {}
  }
}

impl ZObjectTable for MockObjectTable {
  type ZObject = MockObject;
  type DataAccess = MockObjectTableStorage;
  type PropertyTable = MockPropertyTable;

  fn object_with_number(&self, object_number: u16) -> MockObject {
    MockObject { object_number: object_number }
  }
}

impl ZObject for MockObject {
  type DataAccess = MockObjectTableStorage;
  type PropertyTable = MockPropertyTable;

  fn attributes(&self, helper: &MockObjectTableStorage) -> u32 {
    helper.attributes(self.object_number)
  }

  fn set_attributes(&self, attrs: u32, helper: &mut MockObjectTableStorage) {
    helper.set_attributes(self.object_number, attrs);
  }

  fn parent(&self, helper: &MockObjectTableStorage) -> u16 {
    helper.parent(self.object_number)
  }

  fn set_parent(&self, parent: u16, helper: &mut MockObjectTableStorage) {
    helper.set_parent(self.object_number, parent);
  }

  fn sibling(&self, helper: &MockObjectTableStorage) -> u16 {
    helper.sibling(self.object_number)
  }
  fn set_sibling(&self, sibling: u16, helper: &mut MockObjectTableStorage) {
    helper.set_sibling(self.object_number, sibling);
  }

  fn child(&self, helper: &MockObjectTableStorage) -> u16 {
    helper.child(self.object_number)
  }

  fn set_child(&self, child: u16, helper: &mut MockObjectTableStorage) {
    helper.set_child(self.object_number, child);
  }

  fn property_table(&self, helper: &MockObjectTableStorage) -> MockPropertyTable {
    // TODO: make this really work.
    MockPropertyTable { table: HashMap::new() }
  }
}

impl ZPropertyTable for MockPropertyTable {
  type PropertyAccess = bool;
  type Ref = u16;

  fn name_ptr(&self, helper: &bool) -> u16 {
    // TODO: implement a testable version of this.
    32
  }

  // property numbers are 1-31. Returns the size and ptr to the property.
  fn find_property(&self, number: u16, helper: &bool) -> Option<(u16, u16)> {
    // TODO: implement a testable version of this.
    None
  }
}

#[cfg(test)]
mod tests {
  use super::{MockObjectTable, MockObjectTableStorage};
  use zmachine::vm::object_table::{ZObject, ZObjectTable, ZPropertyTable};

  #[test]
  fn test_mock_object_table() {
    // This is a very simple data structure. We only really have to test that
    // objects are mapped to the correct place.
    let storage = MockObjectTableStorage::new();
    let object_table = MockObjectTable::new(&storage);
    let object = object_table.object_with_number(1);
    assert_eq!(1, object.object_number);

    let object = object_table.object_with_number(2);
    assert_eq!(2, object.object_number);

    let object = object_table.object_with_number(3);
    assert_eq!(3, object.object_number);

    let object = object_table.object_with_number(6);
    assert_eq!(6, object.object_number);
  }

  #[test]
  fn test_mock_objects() {
    let mut storage = MockObjectTableStorage::new();
    storage.add_mock_object(0x3456789a, 0x12, 0x13, 0x23);

    let object_table = MockObjectTable::new(&storage);
    let obj = object_table.object_with_number(1);

    assert_eq!(0x3456789a, obj.attributes(&storage));
    assert_eq!(0x12, obj.parent(&storage));
    assert_eq!(0x13, obj.sibling(&storage));
    assert_eq!(0x23, obj.child(&storage));

    obj.set_attributes(0x55667788, &mut storage);
    obj.set_parent(0x11, &mut storage);
    obj.set_sibling(0x77, &mut storage);
    obj.set_child(0xcc, &mut storage);

    assert_eq!(0x55667788, obj.attributes(&storage));
    assert_eq!(0x11, obj.parent(&storage));
    assert_eq!(0x77, obj.sibling(&storage));
    assert_eq!(0xcc, obj.child(&storage));

    // TODO: test property_table().
  }


  // // TODO: fn test_mock_property_table() {}
  // impl MockPropertyTable {
  //   fn new(size: u16) -> MockPropertyTable {
  //     MockPropertyTable { table: HashMap::new() }
  //   }

  //   fn add_property(&mut self, number: u16, size: u16, val: u16) {
  //     self.table.insert(number, MockProperty::new(number, size, val));
  //   }
  // }

  // impl MockProperty {
  //   fn new(number: u16, size: u16, val: u16) -> MockProperty {
  //     MockProperty {
  //       number: number,
  //       size: size,
  //       val: val,
  //     }
  //   }
  // }

  //   impl ZPropertyTable for MockPropertyTable {
  //     type Helper = Self;
  //     type Ref = u16;

  //     fn name_ptr(&self, helper: &Self::Helper) -> u16 {
  //       0
  //     }

  //     fn find_property(&self, number: u16, helper: &Self::Helper) -> Option<(u16, Self::Ref)> {
  //       Some((2, number))
  //     }
  //   }

  //   impl ZPropertyAccess for MockPropertyTable {
  //     type Ref = u16;

  //     fn set_byte_property(&mut self, value: u8, ptr: u16) {
  //       let property = self.table.get_mut(&ptr).unwrap();
  //       property.val = value as u16;
  //     }

  //     fn set_word_property(&mut self, value: u16, ptr: u16) {
  //       let property = self.table.get_mut(&ptr).unwrap();
  //       property.val = value;
  //     }
  //   }

  //   // test set property
  //   fn test_mock_property_table() {
  //     let mut table = MockPropertyTable::new(20);
  //     table.add_property(17, 2, 0x1234);
  //     table.add_property(22, 5, 0);
  //     table.add_property(23, 1, 0x11);

  //     let prop = table.find_property(17, &table);
  //     assert!(prop.is_some());
  //     let prop = table.find_property(22, &table);
  //     assert!(prop.is_some());
  //     let prop = table.find_property(23, &table);
  //     assert!(prop.is_some());

  //     let prop = table.find_property(3, &table);
  //     assert!(prop.is_none());
  //   }
}
