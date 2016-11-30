// YES

use result::Result;
use zmachine::ops::Operand;
use zmachine::vm::{BytePtr, RawPtr, VM};

const ROW1: [char; 26] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
                          'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
const ROW2: [char; 26] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
                          'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
const ROW3: [char; 26] = ['@', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',',
                          '!', '?', '_', '#', '\'', '"', '/', '\\', '-', ':', '(', ')'];

enum TextSource {
  PC,
  Memory(RawPtr, bool),
}

fn decode_text<T>(vm: &mut T, src: TextSource) -> Result<String>
  where T: VM {
  let mut row = ROW1;
  let mut s: String = "".to_string();

  let (text_ptr, from_pc, in_abbrev) = match src {
    TextSource::PC => (None, true, false),
    TextSource::Memory(tp, ia) => (Some(tp), false, ia),
  };

  let mut abbrev_set: Option<u16> = None;

  let mut offset = 0;
  loop {
    // TODO: create a Trait for reading words to simplify this code.
    let word = if from_pc {
      Ok(vm.read_pc_word())
    } else {
      // text_ptr should always be Some() unless from_pc is true.
      let mut ptr = text_ptr.unwrap();
      ptr.inc_by(offset);
      let w = vm.read_memory(ptr);
      offset += 2;
      w
    }?;

    let ch1 = (word >> 10) & 0b11111u16;
    let ch2 = (word >> 5) & 0b11111u16;
    let ch3 = word & 0b11111u16;

    for ch in [ch1, ch2, ch3].into_iter() {
      if let Some(set) = abbrev_set {
        let abbrev_addr = vm.abbrev_addr(set as u8, *ch as u8)?;
        let abbrev = decode_text(vm, TextSource::Memory(abbrev_addr.into(), true))?;
        s.push_str(abbrev.as_str());

        abbrev_set = None;
      } else {
        match *ch {
          0x00u16 => s.push(' '),
          0x01u16...0x03u16 => {
            if in_abbrev {
              panic!("Attempted to read abbrev in abbrev");
            }
            abbrev_set = Some(*ch);
          }
          0x04u16 => row = ROW2,
          0x05u16 => row = ROW3,
          _ => {
            s.push(row[(ch - 6) as usize]);
            row = ROW1;
          }
        }
      }
    }

    if word & 0x8000 != 0 {
      break;
    }
  }
  return Ok(s);
}

pub fn print_0x02<T>(vm: &mut T) -> Result<()>
  where T: VM {
  let s = decode_text(vm, TextSource::PC)?;
  print!("{}", s);
  Ok(())
}

pub fn print_num_0x06<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  // We only care about the first operand.
  let value = operands[0].value(vm)?;
  print!("{}", value);
  Ok(())
}

pub fn print_obj_0x0a<T>(vm: &mut T, operand: Operand) -> Result<()>
  where T: VM {
  // TODO: test print_obj_0x0a
  let object_number = operand.value(vm)?;
  let text_ptr = vm.object_name(object_number)?;
  let str = decode_text(vm, TextSource::Memory(text_ptr, false))?;
  println!("{}", str);
  Ok(())
}

pub fn print_char_0x05<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {
  let ch = operands[0].value(vm)?;
  match ch {
    13 => print!("\n"),
    32...126 => print!("{}", ch as u8 as char),
    _ => {
      println!("UNKNOWN CHARACTER");
    }
  }
  Ok(())
}

pub fn new_line_0x0b<T>(vm: &mut T) -> Result<()>
  where T: VM {
  print!("\n");
  Ok(())
}

#[cfg(test)]
mod test {
  use super::{TextSource, decode_text};
  use zmachine::ops::testvm::TestVM;
  use zmachine::vm::BytePtr;

  #[test]
  fn test_string_from_pc() {
    // f = 0x0b = 0b01011
    // o = 0x14 = 0b10100
    // foo = 0b1 01011 10100 10100
    //     = 0xae94
    let foo = vec![0xae, 0x94];
    let mut vm = TestVM::new();
    vm.set_pcbytes(foo);
    vm.set_pc(0);
    let str = decode_text(&mut vm, TextSource::PC).unwrap();
    assert_eq!("foo", str);
  }

  #[test]
  fn test_short_string_from_pc() {
    // f = 0x0b = 0b01011
    // f = 0b1 01011 00101 00101
    //   = 0xaca5
    let f = vec![0xac, 0xa5];
    let mut vm = TestVM::new();
    vm.set_pcbytes(f);
    vm.set_pc(0);
    let str = decode_text(&mut vm, TextSource::PC).unwrap();
    assert_eq!("f", str);
  }

  #[test]
  fn test_long_string_from_heap() {
    // q = 0x16 = 0b10110
    // u = 0x1a = 0b11010
    // x = 0x1d = 0b11101
    // quux = 0b0 10110 11010 11010, 0b1 11101 00101 00101
    // Leave a couple of zeros at the front to test the ptr code.
    let quux = vec![0x00, 0x00, 0x00, 0x5b, 0x5a, 0xf4, 0xa5];
    let mut vm = TestVM::new();
    vm.set_heap(quux);
    let str = decode_text(&mut vm,
                          TextSource::Memory(BytePtr::new(0x03).into(), false))
      .unwrap();
    assert_eq!("quux", str);
  }

  #[test]
  fn test_char_sets() {
    // q = 0x16 = 0b10110
    // u = 0x1a = 0b11010
    // x = 0x1d = 0b11101
    // ! = 0x14 = 0b10100
    // Quux! = 0x04 0x16 0x1a | 0x1a 0x1d 0x05 | 0x14 0x05 0x05
    //       = 0b0 00100 10110 11010, 0b0 11010 11101 00101, 0b1 10100 00101 00101
    let quux = vec![0x12, 0xda, 0x6b, 0xa5, 0xd0, 0xa5];
    let mut vm = TestVM::new();
    vm.set_pcbytes(quux);
    let str = decode_text(&mut vm, TextSource::PC).unwrap();
    assert_eq!("Quux!", str);
  }

  // TODO: test abbrevs!
  // TODO: test all of the print opcodes after writing an output abstraction.
}
