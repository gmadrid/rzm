use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{PackedAddr, RawPtr, VM, VariableRef};

pub fn call_0x00<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  let result_location = VariableRef::decode(vm.read_pc_byte());
  let return_pc = vm.current_pc();

  let addr_value = operands[0].value(vm)?;

  if addr_value == 0 {
    // TODO: test calling with routine = 0.
    vm.write_variable(result_location, 0);
    return Ok(());
  }

  let packed_addr = PackedAddr::new(addr_value);
  let raw_addr: RawPtr = packed_addr.into();
  vm.set_current_pc(raw_addr.into())?;

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
    let arg_value = operand.value(vm)?;
    argument_values.push(arg_value);
  }

  let num_locals = vm.read_pc_byte();
  vm.new_frame(return_pc, num_locals, result_location)?;
  // The frame is set up. Now initialize the local variables from the code.
  for i in 0..num_locals {
    let local_from_pc = vm.read_pc_word();
    vm.write_local(i, local_from_pc)?;
  }

  // Now, copy in any arguments passed in by call().
  // NOTE: there is a small inefficiency here since locals may be copied twice.
  for (i, val) in argument_values.iter().enumerate() {
    if i >= num_locals as usize {
      break;
    }
    vm.write_local(i as u8, *val)?;
  }
  // for i in 0..num_locals {
  //   println!("Local {} after call: {:x}", i, vm.read_local(i));
  // }
  Ok(())
}

pub fn ret_0x0b<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  let value = operand.value(vm)?;
  vm.ret_value(value)
}

pub fn ret_popped_0x08<T>(vm: &mut T) -> Result<()>
  where T: VM {
  let value = vm.pop_stack()?;
  vm.ret_value(value)
}

pub fn rtrue_0x00<T>(vm: &mut T) -> Result<()>
  where T: VM {
  vm.ret_value(1)
}

pub fn rfalse_0x01<T>(vm: &mut T) -> Result<()>
  where T: VM {
  vm.ret_value(0)
}

// TODO: test this shit.
