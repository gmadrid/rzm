
// #[derive(Debug,Eq,PartialEq)]
// pub enum Operand {
//   LargeConstant(u16),
//   SmallConstant(u8),
// //  Variable(VariableRef),
//   Omitted,
// }

// impl Operand {
//   pub fn value<T>(&self, runner: &mut T) -> u16
//     where T: OpcodeRunner {
//     match *self {
//       Operand::LargeConstant(val) => val,
//       Operand::SmallConstant(val) => val as u16,
//       Operand::Variable(variable) => runner.read_variable(variable),
//       Operand::Omitted => {
//         panic!("Cannot read Omitted operand: {:?}", *self);
//       }
//     }
//   }
// }

// #[cfg(test)]
// pub mod test {
//   use byteorder::{BigEndian, ByteOrder};
//   use super::{OpcodeRunner, VariableRef};

//   pub struct TestRunner {
//     pub heap: Vec<u8>,
//     pub stack: Vec<u16>,
//     pub locals: [u16; 15],
//     pub globals: [u16; 240],
//     pub pc: usize,
//     pub pcbytes: Vec<u8>,
//   }

//   impl TestRunner {
//     pub fn new() -> TestRunner {
//       TestRunner {
//         heap: vec![0; 1000],
//         stack: Vec::new(),
//         locals: [0; 15],
//         globals: [0; 240],
//         pc: 0,
//         pcbytes: Vec::new(),
//       }
//     }

//     pub fn set_result_location(&mut self, location: VariableRef) {
//       self.set_pc_bytes(vec![VariableRef::encode(location)]);
//     }

//     pub fn set_pc_bytes(&mut self, bytes: Vec<u8>) {
//       self.pcbytes = bytes;
//       self.pc = 0;
//     }

//     pub fn set_jump_offset_byte(&mut self, offset: u8, polarity: bool) {
//       let mut byte = 0b01000000u8;
//       if polarity {
//         byte |= 0b10000000;
//       }
//       byte |= offset & 0b00111111;
//       self.set_pc_bytes(vec![byte]);
//     }

//     pub fn set_jump_offset_word(&mut self, offset: i16, polarity: bool) {
//       let mut word = 0u16;
//       if polarity {
//         word |= 0b1000000000000000;
//       }
//       word |= (offset as u16) & 0b0011111111111111;
//       let mut vec = vec![0u8, 0u8];
//       BigEndian::write_u16(vec.as_mut_slice(), word as u16);
//       self.set_pc_bytes(vec);
//     }
//   }

//   impl OpcodeRunner for TestRunner {
//     fn read_pc_byte(&mut self) -> u8 {
//       let val = self.pcbytes[self.pc];
//       self.pc += 1;
//       val
//     }

//     fn read_pc_word(&mut self) -> u16 {
//       let val = BigEndian::read_u16(&self.pcbytes[self.pc..]);
//       self.pc += 2;
//       val
//     }

//     fn current_pc(&self) -> usize {
//       self.pc
//     }

//     fn set_current_pc(&mut self, pc: usize) {
//       self.pc = pc;
//     }

//     fn offset_pc(&mut self, offset: i16) {
//       self.pc = ((self.pc as i32) + (offset as i32)) as usize;
//     }

//     fn read_memory(&self, byteaddress: usize) -> u16 {
//       BigEndian::read_u16(&self.heap[byteaddress..])
//     }

//     fn write_memory(&mut self, byteaddress: usize, val: u16) {
//       BigEndian::write_u16(&mut self.heap[byteaddress..], val)
//     }

//     fn read_memory_u8(&self, byteaddress: usize) -> u8 {
//       self.heap[byteaddress]
//     }

//     fn pop_stack(&mut self) -> u16 {
//       self.stack.pop().unwrap()
//     }

//     fn push_stack(&mut self, val: u16) {
//       self.stack.push(val);
//     }

//     fn new_frame(&mut self, ret_pc: usize, num_locals: u8, result_location: u8) {
//       // not sure what to do with this
//     }

//     fn pop_frame(&mut self, return_val: u16) -> (usize, VariableRef) {
//       // not sure what to do with this either
//       (0, VariableRef::Stack)
//     }

//     fn attributes(&mut self, object_number: u16) -> u32 {
//       unimplemented!()
//     }

//     fn put_property(&mut self, object_index: u16, property_number: u16, value: u16) {
//       unimplemented!()
//     }

//     fn abbrev_addr(&self, abbrev_table: u8, abbrev_index: u8) -> usize {
//       unimplemented!()
//     }

//     fn insert_obj(&mut self, object_index: u16, dest_index: u16) {
//       unimplemented!()
//     }

//     fn read_local(&self, local_idx: u8) -> u16 {
//       self.locals[local_idx as usize]
//     }

//     fn write_local(&mut self, local_idx: u8, val: u16) {
//       self.locals[local_idx as usize] = val;
//     }

//     fn read_global(&self, global_idx: u8) -> u16 {
//       self.globals[global_idx as usize]
//     }

//     fn write_global(&mut self, global_idx: u8, val: u16) {
//       self.globals[global_idx as usize] = val;
//     }

//     fn result_location(&mut self) -> VariableRef {
//       VariableRef::decode(self.read_pc_byte())
//     }
//   }

//   #[test]
//   fn test_variable_decode() {
//     assert_eq!(VariableRef::Stack, VariableRef::decode(0x00));
//     assert_eq!(VariableRef::Local(0), VariableRef::decode(0x01));
//     assert_eq!(VariableRef::Local(5), VariableRef::decode(0x06));
//     assert_eq!(VariableRef::Local(14), VariableRef::decode(0x0f));
//     assert_eq!(VariableRef::Global(0), VariableRef::decode(0x10));
//     assert_eq!(VariableRef::Global(80), VariableRef::decode(0x60));
//     assert_eq!(VariableRef::Global(239), VariableRef::decode(0xff));
//   }
// }
