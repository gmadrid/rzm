use result::Result;
use zmachine::vm::{VM, VariableRef};

mod binop;
mod branch;
mod call;
mod input;
mod load;
mod properties;
mod stackops;
mod text;

#[cfg(test)]
mod testvm;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
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
  // 2do: save_0x05, restore_0x06, restart_0x07
  // 2do: pop_0x09, quit_0x0a, show_status_0x0c, verify_0x0d

  pub use super::call::nop_0x04;
  pub use super::call::ret_popped_0x08;
  pub use super::call::rfalse_0x01;
  pub use super::call::rtrue_0x00;
  pub use super::text::new_line_0x0b;
  pub use super::text::print_0x02;
  pub use super::text::print_ret_0x03;
}

pub mod oneops {
  // 2do: dec_0x06, load_0x0e

  pub use super::branch::get_child_0x02;
  pub use super::branch::get_sibling_0x01;
  pub use super::branch::jump_0x0c;
  pub use super::branch::jz_0x00;
  pub use super::call::ret_0x0b;
  pub use super::load::inc_0x05;
  pub use super::properties::get_parent_0x03;
  pub use super::properties::get_prop_len_0x04;
  pub use super::properties::remove_obj_0x09;
  pub use super::text::print_addr_0x07;
  pub use super::text::print_obj_0x0a;
  pub use super::text::print_paddr_0x0d;
}

pub mod twoops {
  // 2do: or_0x08, get_next_prop_0x13, mod_0x18

  pub use super::binop::add_0x14;
  pub use super::binop::and_0x09;
  pub use super::binop::div_0x17;
  pub use super::binop::mul_0x16;
  pub use super::binop::sub_0x15;
  pub use super::branch::dec_chk_0x04;
  pub use super::branch::inc_chk_0x05;
  pub use super::branch::je_0x01;
  pub use super::branch::jg_0x03;
  pub use super::branch::jin_0x06;
  pub use super::branch::jl_0x02;
  pub use super::branch::test_0x07;
  pub use super::load::loadb_0x10;
  pub use super::load::loadw_0x0f;
  pub use super::load::store_0x0d;
  pub use super::properties::clear_attr_0x0c;
  pub use super::properties::get_prop_0x11;
  pub use super::properties::get_prop_addr_0x12;
  pub use super::properties::insert_obj_0x0e;
  pub use super::properties::set_attr_0x0b;
  pub use super::properties::test_attr_0x0a;
}

pub mod varops {
  // 2do: split_window_0x0a, set_window_0x0b, output_stream_0x13, input_stream_0x14

  pub use super::call::call_0x00;
  pub use super::input::read_0x04;
  pub use super::load::random_0x07;
  pub use super::load::storeb_0x02;
  pub use super::load::storew_0x01;
  pub use super::properties::put_prop_0x03;
  pub use super::stackops::pull_0x09;
  pub use super::stackops::push_0x08;
  pub use super::text::print_char_0x05;
  pub use super::text::print_num_0x06;
}
