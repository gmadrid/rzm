mod dictionary;
mod memory;
mod mm_object_table;
#[cfg(test)]
mod mock_object_table;
mod object_table;
mod pc;
mod ptrs;
mod zreadline;
mod stack;
mod vm;
pub mod zvm;

pub use self::memory::Memory;

pub use self::object_table::{ZObject, ZObjectTable, ZPropertyStorage, ZPropertyTable};
pub use self::ptrs::BytePtr;
pub use self::ptrs::PackedAddr;
pub use self::ptrs::RawPtr;
pub use self::ptrs::WordPtr;
pub use self::vm::VM;
pub use self::vm::VariableRef;

#[cfg(test)]
pub mod test {
  pub use super::mock_object_table::{MockObjectTable, MockObjectTableStorage, MockPropertyTable,
                                     MockPropertyTableStorage};
}
