use result::Result;

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
  fn default_property_value(&self, property_number: u16, access: &Self::DataAccess) -> u16;

  fn remove_object_from_parent(&self,
                               object_number: u16,
                               access: &mut Self::DataAccess)
                               -> Result<()> {
    let obj = self.object_with_number(object_number);
    let parent_number = obj.parent(access);

    if parent_number == 0 {
      // object has no parent, so just return.
      return Ok(());
    }

    let parent_obj = self.object_with_number(parent_number);
    let mut current_number = parent_obj.child(access);
    if current_number == object_number {
      // object is first child
      let new_child = obj.sibling(access);
      parent_obj.set_child(new_child, access);
    } else {
      // object should be in sibling list
      loop {
        assert!(current_number != 0);
        let current_sibling = self.object_with_number(current_number);
        let next_sibling = current_sibling.sibling(access);
        if next_sibling == object_number {
          let new_next_sibling = obj.sibling(access);
          current_sibling.set_sibling(new_next_sibling, access);
          break;
        } else {
          current_number = next_sibling;
        }
      }
    }

    obj.set_sibling(0, access);
    obj.set_parent(0, access);
    return Ok(());
  }

  fn add_object_to_parent(&self,
                          object_number: u16,
                          parent_number: u16,
                          access: &mut Self::DataAccess)
                          -> Result<()> {
    let parent_obj = self.object_with_number(parent_number);
    let obj = self.object_with_number(object_number);

    let new_sibling = parent_obj.child(access);
    obj.set_parent(parent_number, access);
    obj.set_sibling(new_sibling, access);
    parent_obj.set_child(object_number, access);
    Ok(())
  }

  fn insert_obj(&self,
                object_number: u16,
                parent_number: u16,
                access: &mut Self::DataAccess)
                -> Result<()> {
    if object_number == 0 {
      // null object - nothing to do
      return Ok(());
    }

    self.remove_object_from_parent(object_number, access)?;

    if parent_number == 0 {
      return Ok(());
    }

    self.add_object_to_parent(object_number, parent_number, access)
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

#[cfg(test)]
mod tests {
  use super::ZObjectTable;
  use zmachine::vm::mock_object_table::{MockObjectTable, MockObjectTableStorage};

  #[test]
  fn test_insert_0_obj() {
    let mut storage = MockObjectTableStorage::new();
    let table = MockObjectTable::new(&storage);
    table.insert_obj(0, 0, &mut storage).unwrap();
  }

  #[test]
  fn test_remove_has_no_current_parent() {
    let mut storage = MockObjectTableStorage::new();
    storage.add_mock_object(0x11223344, 0, 0, 0);
    let storage2 = storage.clone();

    let table = MockObjectTable::new(&storage);

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

    let table = MockObjectTable::new(&storage);
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

    let table = MockObjectTable::new(&mut storage);
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

    let table = MockObjectTable::new(&mut storage);
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
