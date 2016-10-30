use result::{Error, Result};
use zmachine::opcodes::{OpcodeRunner, Operand, Operands};

fn byteaddress_from_packed_addr(packed_addr: u16) -> usize {
  2 * packed_addr as usize
}

pub fn call_0x00<T>(runner: &mut T, operands: Operands) -> Result<()>
  where T: OpcodeRunner {
  if let Operands::Var(arr) = operands {
    let result_location = runner.read_pc_byte();
    let return_pc = runner.current_pc();

    let packed_addr = arr[0].value(runner);
    let byteaddress = byteaddress_from_packed_addr(packed_addr);
    runner.set_current_pc(byteaddress);

    // Need to read the argument values _before_ creating the new frame.
    let mut argument_values = Vec::<u16>::with_capacity(3);
    for (i, operand) in arr.into_iter().enumerate() {
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
    for i in 0..num_locals {
      println!("Local {} after call: {:x}", i, runner.read_local(i));
    }
  } else {
    return Err(Error::BadOperands("call must take VAR args.".to_string(), operands));
  }
  Ok(())
}
