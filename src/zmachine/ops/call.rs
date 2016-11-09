// YES

use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{PackedAddr, RawPtr, VM, VariableRef};

pub fn call_0x00<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  let result_location = VariableRef::decode(vm.read_pc_byte());
  let return_pc = vm.current_pc();

  let addr_value = try!(operands[0].value(vm));
  let packed_addr = PackedAddr::new(addr_value);
  let raw_addr: RawPtr = packed_addr.into();
  try!(vm.set_current_pc(raw_addr.into()));

  // Need to read the argument values _before_ creating the new frame.
  let mut argument_values = Vec::<u16>::with_capacity(3);
  for (i, operand) in operands.into_iter().enumerate() {
    if i == 0 {
      // Skip call address
      continue;
    }
    if Operand::Omitted == *operand {
      break;
    }
    let arg_value = try!(operand.value(vm));
    argument_values.push(arg_value);
  }

  let num_locals = vm.read_pc_byte();
  try!(vm.new_frame(return_pc, num_locals, result_location));
  // The frame is set up. Now initialize the local variables from the code.
  for i in 0..num_locals {
    let local_from_pc = vm.read_pc_word();
    try!(vm.write_local(i, local_from_pc));
  }

  // Now, copy in any arguments passed in by call().
  // NOTE: there is a small inefficiency here since locals may be copied twice.
  for (i, val) in argument_values.iter().enumerate() {
    if i >= num_locals as usize {
      break;
    }
    try!(vm.write_local(i as u8, *val));
  }
  // for i in 0..num_locals {
  //   println!("Local {} after call: {:x}", i, vm.read_local(i));
  // }
  Ok(())
}

pub fn ret_0x0b<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  let value = try!(operand.value(vm));
  vm.ret_value(value)
}

pub fn rtrue_0x00<T>(vm: &mut T) -> Result<()>
  where T: VM {
  vm.ret_value(1)
}

// TODO: test this shit.
