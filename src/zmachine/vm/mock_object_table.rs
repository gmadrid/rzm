use zmachine::vm::object_table;

#[derive(Debug,Clone)]
struct MockObjectTable {
  objects: Vec<MockObjectRep>,
}

struct MockObject {
  index: u16,
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
}
