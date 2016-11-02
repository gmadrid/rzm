use result::Result;
use zmachine::opcodes::{OpcodeRunner, Operand};

const ROW1: [char; 26] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
                          'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
const ROW2: [char; 26] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
                          'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
const ROW3: [char; 26] = ['@', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',',
                          '!', '?', '_', '#', '\'', '"', '/', '\\', '-', ':', '(', ')'];

enum TextSource {
  PC,
  Memory(usize, bool), // byteaddress, in_abbrev
}

fn decode_text<T>(runner: &mut T, src: TextSource) -> String
  where T: OpcodeRunner {
  let mut row = ROW1;
  let mut s: String = "".to_string();

  let (mut text_addr, in_abbrev) = match src {
    TextSource::Memory(ta, ia) => (ta, ia),
    TextSource::PC => (0, false),
  };
  let mut abbrev_set: Option<u16> = None;

  loop {
    let mut word = if text_addr > 0 {
      let w = runner.read_memory(text_addr);
      text_addr += 2;
      w
    } else {
      runner.read_pc_word()
    };

    let ch1 = (word >> 10) & 0b11111u16;
    let ch2 = (word >> 5) & 0b11111u16;
    let ch3 = word & 0b11111u16;

    for ch in [ch1, ch2, ch3].into_iter() {
      if let Some(set) = abbrev_set {
        let abbrev_addr = runner.abbrev_addr(set as u8, *ch as u8);
        let abbrev = decode_text(runner, TextSource::Memory(abbrev_addr, true));
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
  return s;
}

pub fn print_0x02<T>(runner: &mut T) -> Result<()>
  where T: OpcodeRunner {
  let s = decode_text(runner, TextSource::PC);
  print!("{}", s);
  Ok(())
}

pub fn print_num_0x06<T>(runner: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: OpcodeRunner {
  // We only care about the first operand.
  let value = operands[0].value(runner);
  print!("{}", value);
  Ok(())
}

pub fn print_char_0x05<T>(runner: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: OpcodeRunner {
  let ch = operands[0].value(runner);
  match ch {
    13 => print!("\n"),
    32...126 => print!("{}", ch as u8 as char),
    _ => {
      println!("UNKNOWN CHARACTER");
    }
  }
  Ok(())
}

pub fn new_line_0x0b<T>(runner: &mut T) -> Result<()>
  where T: OpcodeRunner {
  print!("\n");
  Ok(())
}
