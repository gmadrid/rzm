use result::Result;
use zmachine::vm::BytePtr;

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
  type ZObject: ZObject<PropertyTable = Self::PropertyTable>;
  type PropertyTable: ZPropertyTable;

  fn object_with_number(&self, object_number: u16) -> Self::ZObject;
  fn default_property_value(&self, property_number: u16) -> u16;

  fn remove_object_from_parent(&self, object_number: u16) -> Result<()> {
    let obj = self.object_with_number(object_number);
    let parent_number = obj.parent();

    if parent_number == 0 {
      // object has no parent, so just return.
      return Ok(());
    }

    let parent_obj = self.object_with_number(parent_number);
    let mut current_number = parent_obj.child();
    if current_number == object_number {
      // object is first child
      let new_child = obj.sibling();
      parent_obj.set_child(new_child);
    } else {
      // object should be in sibling list
      loop {
        assert!(current_number != 0);
        let current_sibling = self.object_with_number(current_number);
        let next_sibling = current_sibling.sibling();
        if next_sibling == object_number {
          let new_next_sibling = obj.sibling();
          current_sibling.set_sibling(new_next_sibling);
          break;
        } else {
          current_number = next_sibling;
        }
      }
    }

    obj.set_sibling(0);
    obj.set_parent(0);
    return Ok(());
  }

  fn add_object_to_parent(&self, object_number: u16, parent_number: u16) -> Result<()> {
    let parent_obj = self.object_with_number(parent_number);
    let obj = self.object_with_number(object_number);

    let new_sibling = parent_obj.child();
    obj.set_parent(parent_number);
    obj.set_sibling(new_sibling);
    parent_obj.set_child(object_number);
    Ok(())
  }

  fn insert_obj(&self, object_number: u16, parent_number: u16) -> Result<()> {
    if object_number == 0 {
      // null object - nothing to do
      return Ok(());
    }

    self.remove_object_from_parent(object_number)?;

    if parent_number == 0 {
      return Ok(());
    }

    self.add_object_to_parent(object_number, parent_number)
  }
}

pub trait ZObject {
  type PropertyTable;

  fn attributes(&self) -> u32;
  fn set_attributes(&self, attrs: u32);
  fn parent(&self) -> u16;
  fn set_parent(&self, parent: u16);
  fn sibling(&self) -> u16;
  fn set_sibling(&self, sibling: u16);
  fn child(&self) -> u16;
  fn set_child(&self, child: u16);
  fn property_table(&self) -> Self::PropertyTable;
}

pub trait ZPropertyStorage {
  fn byte_property(&self, ptr: BytePtr) -> u16;
  fn word_property(&self, ptr: BytePtr) -> u16;
  fn set_byte_property(&mut self, value: u8, ptr: BytePtr);
  fn set_word_property(&mut self, value: u16, ptr: BytePtr);
}

pub trait ZPropertyTable {
  type Storage: ZPropertyStorage;

  fn storage(&self) -> Self::Storage;

  fn name_ptr(&self) -> BytePtr;
  // property numbers are 1-31. Returns the size and ptr to the property.
  fn find_property(&self, number: u16) -> Option<(u16, BytePtr)>;

  // given a property number, return the next number of the property in the table.
  // Special cases:
  //   0: return the first property,
  //   non-existing property: panic! (according to spec)
  //   no next property: return 0
  fn next_property(&self, number: u16) -> u16;

  // TODO: test set_property
  fn set_property(&self, number: u16, value: u16) {
    if let Some((size, ptr)) = self.find_property(number) {
      match size {
        1 => self.storage().set_byte_property(value as u8, ptr),
        2 => self.storage().set_word_property(value, ptr),
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

#[cfg(test)]
mod tests {
  use super::ZObjectTable;
  use zmachine::vm::mock_object_table::{MockObjectTable, MockObjectTableStorage};

  #[test]
  fn test_insert_0_obj() {
    let mut storage = MockObjectTableStorage::new();
    let table = MockObjectTable::new();
    table.insert_obj(0, 0, &mut storage).unwrap();
  }

  #[test]
  fn test_remove_has_no_current_parent() {
    let mut storage = MockObjectTableStorage::new();
    storage.add_mock_object(0x11223344, 0, 0, 0);
    let storage2 = storage.clone();

    let table = MockObjectTable::new();

    table.remove_object_from_parent(1, &mut storage);

    assert_eq!(storage, storage2);
  }

  #[test]
  fn test_remove_obj_is_child() {
    let mut storage = MockObjectTableStorage::new();
    // 1
    // +-2
    //   3
    //   4
    storage.add_mock_object(0x11223344, 0, 0, 2); // 1
    storage.add_mock_object(0x11223344, 1, 3, 0); // 2
    storage.add_mock_object(0x11223344, 1, 4, 0); // 3
    storage.add_mock_object(0x11223344, 1, 0, 0); // 4

    let table = MockObjectTable::new();
    table.remove_object_from_parent(2, &mut storage).unwrap();

    let mut storage2 = MockObjectTableStorage::new();
    // 1
    // +-3
    //   4
    // 2
    storage2.add_mock_object(0x11223344, 0, 0, 3); // 1
    storage2.add_mock_object(0x11223344, 0, 0, 0); // 2
    storage2.add_mock_object(0x11223344, 1, 4, 0); // 3
    storage2.add_mock_object(0x11223344, 1, 0, 0); // 4

    assert_eq!(storage, storage2);
  }

  #[test]
  fn test_remove_obj_is_in_sibling_list() {
    let mut storage = MockObjectTableStorage::new();
    // 1
    // +-2
    //   3
    //   4
    storage.add_mock_object(0x11223344, 0, 0, 2); // 1
    storage.add_mock_object(0x11223344, 1, 3, 0); // 2
    storage.add_mock_object(0x11223344, 1, 4, 0); // 3
    storage.add_mock_object(0x11223344, 1, 0, 0); // 4

    let table = MockObjectTable::new();
    table.remove_object_from_parent(3, &mut storage).unwrap();

    let mut storage2 = MockObjectTableStorage::new();
    // 1
    // +-2
    //   4
    // 3
    storage2.add_mock_object(0x11223344, 0, 0, 2); // 1
    storage2.add_mock_object(0x11223344, 1, 4, 0); // 2
    storage2.add_mock_object(0x11223344, 0, 0, 0); // 3
    storage2.add_mock_object(0x11223344, 1, 0, 0); // 4

    assert_eq!(storage, storage2);
  }

  #[test]
  fn test_add_obj() {
    let mut storage = MockObjectTableStorage::new();
    storage.add_mock_object(0x11223344, 0, 0, 0); // 1
    storage.add_mock_object(0x11223344, 0, 0, 0); // 2
    storage.add_mock_object(0x11223344, 0, 0, 0); // 3
    storage.add_mock_object(0x11223344, 0, 0, 0); // 4
    storage.add_mock_object(0x11223344, 0, 0, 0); // 5

    let table = MockObjectTable::new();
    table.add_object_to_parent(2, 1, &mut storage);
    table.add_object_to_parent(3, 1, &mut storage);
    table.add_object_to_parent(4, 1, &mut storage);

    let mut storage2 = MockObjectTableStorage::new();
    storage2.add_mock_object(0x11223344, 0, 0, 4); // 1
    storage2.add_mock_object(0x11223344, 1, 0, 0); // 2
    storage2.add_mock_object(0x11223344, 1, 2, 0); // 3
    storage2.add_mock_object(0x11223344, 1, 3, 0); // 4
    storage2.add_mock_object(0x11223344, 0, 0, 0); // 5

    assert_eq!(storage, storage2);
  }

  #[test]
  fn test_insert_obj_zeros() {
    // TODO: write some tests for insert_object.
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


}
