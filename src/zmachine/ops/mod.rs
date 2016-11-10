use result::Result;
use zmachine::vm::{VM, VariableRef};

mod binop;
mod branch;
mod call;
mod load;
mod properties;
mod text;

#[cfg(test)]
mod testvm;

#[derive(Debug,Eq,PartialEq)]
pub enum Operand {
  LargeConstant(u16),
  SmallConstant(u8),
  Variable(VariableRef),
  Omitted,
}

impl Operand {
  pub fn value<T>(&self, runner: &mut T) -> Result<u16>
    where T: VM {
    match *self {
      Operand::LargeConstant(val) => Ok(val),
      Operand::SmallConstant(val) => Ok(val as u16),
      Operand::Variable(variable) => runner.read_variable(variable),
      Operand::Omitted => {
        panic!("Cannot read Omitted operand: {:?}", *self);
      }
    }
  }
}

pub mod zeroops {
  pub use super::call::rtrue_0x00;
  pub use super::text::new_line_0x0b;
  pub use super::text::print_0x02;
}

pub mod oneops {
  pub use super::branch::jump_0x0c;
  pub use super::branch::jz_0x00;
  pub use super::call::ret_0x0b;
}

pub mod twoops {
  pub use super::binop::add_0x14;
  pub use super::binop::and_0x09;
  pub use super::binop::sub_0x15;
  pub use super::branch::inc_chk_0x05;
  pub use super::branch::je_0x01;
  pub use super::load::loadb_0x10;
  pub use super::load::loadw_0x0f;
  pub use super::load::store_0x0d;
  pub use super::properties::insert_obj_0x0e;
  pub use super::properties::test_attr_0x0a;
}

pub mod varops {
  pub use super::call::call_0x00;
  pub use super::load::storew_0x01;
  pub use super::properties::put_prop_0x03;
  pub use super::text::print_char_0x05;
  pub use super::text::print_num_0x06;
}
