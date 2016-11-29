use result::Result;
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
  type ZObject: ZObject<DataAccess = Self::DataAccess, PropertyTable = Self::PropertyTable>;
  type DataAccess;
  type PropertyTable: ZPropertyTable;

  fn object_with_number(&self, object_number: u16) -> Self::ZObject;

  fn insert_obj(&mut self, object_number: u16, parent_number: u16) -> Result<()> {
    unimplemented!()
  }
}

pub trait ZObject {
  type DataAccess;
  type PropertyTable;

  fn attributes(&self, helper: &Self::DataAccess) -> u32;
  fn set_attributes(&self, attrs: u32, helper: &mut Self::DataAccess);
  fn parent(&self, helper: &Self::DataAccess) -> u16;
  fn set_parent(&self, parent: u16, helper: &mut Self::DataAccess);
  fn sibling(&self, helper: &Self::DataAccess) -> u16;
  fn set_sibling(&self, sibling: u16, helper: &mut Self::DataAccess);
  fn child(&self, helper: &Self::DataAccess) -> u16;
  fn set_child(&self, child: u16, helper: &mut Self::DataAccess);
  fn property_table(&self, helper: &Self::DataAccess) -> Self::PropertyTable;
}

pub trait ZPropertyAccess {
  type Ref;

  fn set_byte_property(&mut self, value: u8, ptr: Self::Ref);
  fn set_word_property(&mut self, value: u16, ptr: Self::Ref);
}

pub trait ZPropertyTable {
  type PropertyAccess;
  type Ref;

  fn name_ptr(&self, helper: &Self::PropertyAccess) -> Self::Ref;
  // property numbers are 1-31. Returns the size and ptr to the property.
  fn find_property(&self, number: u16, helper: &Self::PropertyAccess) -> Option<(u16, Self::Ref)>;

  // TODO: test set_property
  fn set_property(&self, number: u16, value: u16, helper: &mut Self::PropertyAccess)
    where Self::PropertyAccess: ZPropertyAccess<Ref = Self::Ref> {
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

#[cfg(testt)]
mod tests {

  use super::{MemoryMappedObject, MemoryMappedObjectTable, MemoryMappedPropertyTable, ZObject,
              ZObjectTable, ZPropertyAccess, ZPropertyTable};
  use zmachine::vm::{BytePtr, Memory};

  #[cfg(test)]
  use zmachine::vm::mock_object_table;

  impl ZObjectTable for MockObjectTable {
    type ZObject = MockObject;
    type DataAccess = Self;
    type PropertyTable = MockPropertyTable;

    fn object_with_number(&self, object_number: u16) -> MockObject {
      self.objects[object_number as usize - 1]
    }
  }

  impl ZObject for MockObject {
    type DataAccess = MockObjectTable;
    type PropertyTable = MockPropertyTable;

    fn attributes(&self, helper: &MockObjectTable) -> u32 {
      helper.attributes
    }
    fn set_attributes(&self, attrs: u32, helper: &mut MockObjectTable) {
      helper.attributes = attrs;
    }
    fn parent(&self, helper: &MockObjectTable) -> u16 {
      helper.parent
    }
    fn set_parent(&self, parent: u16, helper: &mut MockObjectTable) {
      helper.parent = parent;
    }
    fn sibling(&self, helper: &MockObjectTable) -> u16 {
      helper.sibling
    }
    fn set_sibling(&self, sibling: u16, helper: &mut MockObjectTable) {
      helper.sibling = sibling;
    }
    fn child(&self, helper: &MockObjectTable) -> u16 {
      helper.child
    }
    fn set_child(&self, child: u16, helper: &mut MockObjectTable) {
      helper.child = child;
    }
    fn property_table(&self, helper: &MockObjectTable) -> MockPropertyTable {
      MockPropertyTable::new(0)
    }
  }

  #[test]
  fn test_insert_0_obj() {
    let mut table = MockObjectTable::new();
    table.insert_obj(0, 0).unwrap();
  }

  #[test]
  fn test_insert_obj_zeros() {
    // 0, 0, should be okay, but do nothing.

    // 0, 1, should be okay, but do nothing.

    // 1, 0, should remove from parent:
    // * obj 1 has no parent, is okay, but does nothing.
    // * obj 1 has a parent and is its child.
    // * obj 1 has a parent and is in the sibling list of its child.
  }

  fn test_insert_obj() {
    // 1 -> 2 where O x D:
    // O: obj has no parent, no sibs
    //    obj has parent, no sibs
    //    obj is child, no sibs
    //    obj is child, has sibs
    //    obj is not child, last sib
    //    obj is not child, middle sib
    // D: dest has no children
    //    dest has one child
    //    dest has many children
    //    dest is already parent of obj
    //    dest already in objs sibling list
  }

  impl MockPropertyTable {
    fn new(size: u16) -> MockPropertyTable {
      MockPropertyTable { table: HashMap::new() }
    }

    fn add_property(&mut self, number: u16, size: u16, val: u16) {
      self.table.insert(number, MockProperty::new(number, size, val));
    }
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
