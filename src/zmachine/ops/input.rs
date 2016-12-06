use result::Result;
use std::cmp;
use std::io::{self, Write};
use zmachine::ops::Operand;
use zmachine::ops::text::decode_at;
use zmachine::vm::{BytePtr, RawPtr, VM};

enum CharType {
  WhiteSpace,
  Separator,
  WordChar,
}

impl CharType {
  fn char_type(ch: char) -> CharType {
    if ch == ' ' || ch == '\t' || ch == '\n' {
      CharType::WhiteSpace
    } else if ch == '.' || ch == ',' {
      // Stop hard-coding the separator characters.
      CharType::Separator
    } else {
      CharType::WordChar
    }
  }
}

#[derive(Clone, Copy, Debug)]
struct Token {
  ptr: Option<BytePtr>,
  offset: u8,
  len: u8,
}

struct Tokenizer {
  str: String,
  in_word: bool,
  word_start: u8,
  word_length: u8,
  tokens: Vec<Token>,
}

impl Tokenizer {
  fn new() -> Tokenizer {
    Tokenizer {
      str: String::new(),
      in_word: false,
      word_start: 0,
      word_length: 0,
      tokens: Vec::new(),
    }
  }

  fn tokenize<T>(&mut self, vm: &mut T, s: String)
    where T: VM {
    self.str = s.clone();

    for (offset, ch) in s.chars().enumerate() {
      let ctype = CharType::char_type(ch);
      match ctype {
        CharType::WhiteSpace => self.handle_whitespace(vm),
        CharType::Separator => self.handle_separator(vm, offset as u8),
        CharType::WordChar => self.handle_wordchar(offset as u8),
      }
    }
  }

  fn tokens(self) -> Vec<Token> {
    self.tokens
  }

  fn handle_wordchar(&mut self, offset: u8) {
    if self.in_word {
      self.word_length += 1
    } else {
      self.in_word = true;
      self.word_start = offset + 1;  // Account for the length byte.
      self.word_length = 1;
    }
  }

  fn handle_separator<T>(&mut self, vm: &mut T, offset: u8)
    where T: VM {
    self.maybe_push_word_token(vm);
    self.tokens.push(Token {
      ptr: None,
      offset: offset + 1, // Account for the length byte.
      len: 1,
    })
  }

  fn handle_whitespace<T>(&mut self, vm: &mut T)
    where T: VM {
    // Finish off a word token if we're in one, then ignore whitespace.
    self.maybe_push_word_token(vm);
  }

  fn lookup_in_dictionary<T>(&self, vm: &mut T) -> Option<BytePtr>
    where T: VM {
    // TODO: make this a binary search.
    // Truncate the match string to 6 characters to match what is in the dict.
    let len = cmp::min(self.word_length, 6);
    let str = &self.str[(self.word_start - 1) as usize..(self.word_start - 1 + len) as usize];
    for i in 0..vm.num_dict_entries() {
      let entry_number = i + 1;
      let entry_ptr = vm.dict_entry(entry_number);
      let dict_str = decode_at(vm, entry_ptr).unwrap();
      if str == dict_str.as_str() {
        return Some(entry_ptr);
      }
    }
    None
  }

  fn maybe_push_word_token<T>(&mut self, vm: &mut T)
    where T: VM {
    if self.in_word {
      let ptr = self.lookup_in_dictionary(vm);
      self.tokens.push(Token {
        ptr: ptr,
        offset: self.word_start,
        len: self.word_length,
      });
      self.in_word = false;
    }
  }
}

pub fn read_0x04<T>(vm: &mut T, operands: [Operand; 4]) -> Result<()>
  where T: VM {

  // TODO: put the status line in there.
  //          vm.display_status_line();
  // TODO: separators not getting tokenized correctly. "hi, sailor"
  io::stdout().flush()?;

  let s = io::stdin();
  let mut buf = String::new();
  s.read_line(&mut buf)?;
  buf = buf.to_lowercase();

  let tbuf = BytePtr::new(operands[0].value(vm)?);
  let pbuf = BytePtr::new(operands[1].value(vm)?);

  let tbuf_len = (vm.read_memory_u8(tbuf)? + 1) as usize;

  let mut ptr = tbuf.inc_by(1);
  for (pos, ch) in buf.chars().enumerate() {
    if ch == '\n' {
      // Right now, I don't think I need to do anything, since the CR is getting printed when typed.
    }
    if pos >= tbuf_len - 2 || ch == '\n' {
      // null-terminated
      vm.write_memory_u8(ptr, 0)?;
      break;
    }
    vm.write_memory_u8(ptr, ch as u8)?;
    ptr = ptr.inc_by(1);
  }

  // TODO: split on the ., as well..
  let mut tokenizer = Tokenizer::new();
  tokenizer.tokenize(vm, buf);
  let tokens = tokenizer.tokens();

  // TODO: add code to respect the end of the tbuf and pbuf.
  let mut ptr = pbuf.inc_by(1);
  vm.write_memory_u8(ptr, tokens.len() as u8)?;
  ptr = ptr.inc_by(1);
  for token in tokens {
    let val = token.ptr.map(|p| RawPtr::from(p).into()).unwrap_or(0usize) as u16;
    vm.write_memory(ptr, val)?;
    ptr = ptr.inc_by(2);
    vm.write_memory_u8(ptr, token.len)?;
    ptr = ptr.inc_by(1);
    vm.write_memory_u8(ptr, token.offset)?;
    ptr = ptr.inc_by(1);
  }
  Ok(())
}
