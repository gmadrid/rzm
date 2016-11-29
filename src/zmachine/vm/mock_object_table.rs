use std::collections::HashMap;

use zmachine::vm::object_table::{ZObject, ZObjectTable, ZPropertyTable};

#[derive(Debug,Clone)]
struct MockObjectTable {
  objects: Vec<MockObjectRep>,
}

struct MockObject {
  object_number: u16,
}

#[derive(Debug,Clone)]
struct MockObjectRep {
  attributes: u32,
  parent: u16,
  sibling: u16,
  child: u16,
}

impl MockObjectTable {
  fn new() -> MockObjectTable {
    MockObjectTable { objects: Vec::new() }
  }

  fn add_mock_object(&mut self, attributes: u32, parent: u16, sibling: u16, child: u16) {
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

struct MockPropertyTable {
  table: HashMap<u16, MockProperty>,
}

#[derive(Clone, Copy)]
struct MockProperty {
  number: u16,
  size: u16,
  val: u16,
}


impl ZObjectTable for MockObjectTable {
  type ZObject = MockObject;
  type DataAccess = Self;
  type PropertyTable = MockPropertyTable;

  fn object_with_number(&self, object_number: u16) -> MockObject {
    MockObject { object_number: object_number }
  }
}

impl ZObject for MockObject {
  type DataAccess = MockObjectTable;
  type PropertyTable = MockPropertyTable;

  fn attributes(&self, helper: &MockObjectTable) -> u32 {
    helper.attributes(self.object_number)
  }

  fn set_attributes(&self, attrs: u32, helper: &mut MockObjectTable) {
    helper.set_attributes(self.object_number, attrs);
  }

  fn parent(&self, helper: &MockObjectTable) -> u16 {
    helper.parent(self.object_number)
  }

  fn set_parent(&self, parent: u16, helper: &mut MockObjectTable) {
    helper.set_parent(self.object_number, parent);
  }

  fn sibling(&self, helper: &MockObjectTable) -> u16 {
    helper.sibling(self.object_number)
  }
  fn set_sibling(&self, sibling: u16, helper: &mut MockObjectTable) {
    helper.set_sibling(self.object_number, sibling);
  }

  fn child(&self, helper: &MockObjectTable) -> u16 {
    helper.child(self.object_number)
  }

  fn set_child(&self, child: u16, helper: &mut MockObjectTable) {
    helper.set_child(self.object_number, child);
  }

  fn property_table(&self, helper: &MockObjectTable) -> MockPropertyTable {
    // TODO: make this really work.
    MockPropertyTable { table: HashMap::new() }
  }
}

impl ZPropertyTable for MockPropertyTable {
  type PropertyAccess = bool;
  type Ref = u16;

  fn name_ptr(&self, helper: &bool) -> u16 {
    32
  }

  // property numbers are 1-31. Returns the size and ptr to the property.
  fn find_property(&self, number: u16, helper: &bool) -> Option<(u16, u16)> {
    None
  }
}
