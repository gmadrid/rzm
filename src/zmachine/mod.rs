use result::Result;
use std::io::Read;

mod header;
mod memory;
mod pc;

use self::header::Header;
use self::memory::Memory;
use self::pc::PC;

// const HEADER_SIZE: usize = 64;
// const STACK_SIZE: usize = 61440;

pub struct ZMachine {
  memory: Memory,
  pc: PC, /* header: [u8; HEADER_SIZE],
           *
           * pc: usize,
           * stack: [u8; STACK_SIZE],
           * sp: u16,
           *
           * smem_index: u16,
           * hmem_index: u16,
           *
           * memory: Vec<u8>, */
}

impl ZMachine {
  pub fn from_reader<T: Read>(mut reader: T) -> Result<ZMachine> {
    let mut zbytes = Vec::<u8>::new();
    try!(reader.read_to_end(&mut zbytes));

    let memory: Memory = From::from(zbytes);
    let mut pc: Option<PC> = None;
    {
      let header: Header = From::from(&memory);
      pc = Some(PC::new(header.starting_pc()));
    }
    // let mut zmachine = ZMachine {
    //   header: [0; HEADER_SIZE],
    //   pc: 0,
    //   stack: [0; STACK_SIZE],
    //   sp: 0,
    //   smem_index: 0,
    //   hmem_index: 0,
    //   memory: Vec::new(),
    // };
    // try!(reader.read_exact(&mut zmachine.header));

    // let num_bytes_read = try!(reader.read_to_end(&mut zmachine.memory));
    // let num_bytes_expected = BigEndian::read_u16(&zmachine.header[0x1a..]) as u32 * 2;
    // if num_bytes_expected > 0 && num_bytes_read < num_bytes_expected as usize {
    //   panic!("File too short. Header expects at least {:x} bytes",
    //          num_bytes_expected);
    // }

    // zmachine.reset_interpreter_flags();
    // zmachine.hmem_index = BigEndian::read_u16(&zmachine.header[0x04..]);
    // //    zmachine.pc = BigEndian::read_u16(&zmachine.header[0x06..]);
    // zmachine.smem_index = BigEndian::read_u16(&zmachine.header[0x0e..]);

    Ok(ZMachine {
      memory: memory,
      pc: pc.unwrap(),
    })
  }

  pub fn run(&mut self) -> Result<()> {
    loop {
      //      try!(self.process_opcode());
      break;
    }
    Ok(())
  }

  // fn process_opcode(&mut self) -> Result<()> {
  //   let first_byte = self.memory[(self.pc - 64) as usize];
  //   println!("first byte: {:x}/{:b}", first_byte, first_byte);
  //   self.pc += 1;
  //   match first_byte {
  //     0xe0...0xff => try!(self.process_var_opcode(first_byte)),
  //     _ => {
  //       println!("Unknown opcode: {:?}", first_byte);
  //       ()  // TODO: This should be an error
  //     }
  //   }
  //   Ok(())
  // }

  // fn process_var_opcode(&mut self, first_byte: u8) -> Result<()> {
  //   let next_byte = self.memory[(self.pc - 64) as usize];
  //   self.pc += 1;
  //   println!("next_byte: {:b}", next_byte);
  //   let address = BigEndian::read_u16(&self.memory[(self.pc as usize)..]);
  //   println!("address: {:x}", address * 2);
  //   self.pc += 2;
  //   let arg1 = BigEndian::read_u16(&self.memory[(self.pc as usize)..]);
  //   self.pc += 2;
  //   let arg2 = BigEndian::read_u16(&self.memory[(self.pc as usize)..]);
  //   self.pc += 2;
  //   println!("arg1: {:x}", arg1);
  //   println!("arg2: {:x}", arg2);
  //   Ok(())
  // }

  // fn reset_interpreter_flags(&mut self) {
  //   // The interpreter sets flags in the header to express its capabilities to the game.
  //   let flag1_mask = !0b01110000;  // no status line, no split screen, fixed-width font
  //   self.header[0x01] &= flag1_mask;
  // }
  // pub fn machine_version(&self) -> u8 {
  //   self.header[0]
  // }
}
