use result::{Error, Result};
use super::ret_value;
use zmachine::opcodes::{OpcodeRunner, Operand};

fn byteaddress_from_packed_addr(packed_addr: u16) -> usize {
  2 * packed_addr as usize
}

pub fn call_0x00<T>(runner: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: OpcodeRunner {
  let result_location = runner.read_pc_byte();
  let return_pc = runner.current_pc();

  let packed_addr = operands[0].value(runner);
  let byteaddress = byteaddress_from_packed_addr(packed_addr);
  runner.set_current_pc(byteaddress);

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
    argument_values.push(operand.value(runner));
  }

  let num_locals = runner.read_pc_byte();
  runner.new_frame(return_pc, num_locals, result_location);

  // The frame is set up. Now initialize the local variables from the code.
  for i in 0..num_locals {
    let local_from_pc = runner.read_pc_word();
    runner.write_local(i, local_from_pc);
  }

  // Now, copy in any arguments passed in by call().
  // NOTE: there is a small inefficiency here since locals may be copied twice.
  for (i, val) in argument_values.iter().enumerate() {
    if i >= num_locals as usize {
      break;
    }
    runner.write_local(i as u8, *val);
  }
  // for i in 0..num_locals {
  //   println!("Local {} after call: {:x}", i, runner.read_local(i));
  // }
  Ok(())
}

pub fn ret_0x0b<T>(runner: &mut T, operand: Operand) -> Result<()>
  where T: OpcodeRunner {
  let value = operand.value(runner);
  //  println!("ret {:?}: {:x}", operand, value);
  ret_value(runner, value)
}
