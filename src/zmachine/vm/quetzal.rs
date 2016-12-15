use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use result::Result;
use std::io::Write;
use zmachine::vm::memory::Memory;
use zmachine::vm::stack::Stack;

struct Chunk {
  id: u32,
  start_offset: usize,
}

// Make this work with a &str.
pub fn newId(ch1: char, ch2: char, ch3: char, ch4: char) -> u32 {
  let bytes = [ch1 as u8, ch2 as u8, ch3 as u8, ch4 as u8];
  BigEndian::read_u32(&bytes)
}

impl Chunk {
  fn start(id: u32, bytes: &mut Vec<u8>) -> Result<Chunk> {
    info!(target: "foo", "start: {}", bytes.len());
    bytes.write_u32::<BigEndian>(id)?;
    bytes.write_u32::<BigEndian>(0)?;
    info!(target: "foo", "after: {}", bytes.len());
    Ok(Chunk {
      id: id,
      start_offset: bytes.len() - 8,
    })
  }

  fn end(self, bytes: &mut Vec<u8>) {
    // Subtract 8 to account for the id and size which are not counted in the chunk size.
    let used_bytes = bytes.len() - self.start_offset - 8;
    BigEndian::write_u32(&mut bytes[self.start_offset + 4..], used_bytes as u32);

    if used_bytes % 2 == 1 {
      // Spec says to pad the chunk if odd-length.
      bytes.write_u8(0);
    }
  }
}

pub struct Quetzal {
  bytes: Vec<u8>,
}

impl Quetzal {
  fn write_header(&mut self, memory: &Memory) -> Result<()> {
    let chunk = Chunk::start(newId('I', 'F', 'h', 'd'), &mut self.bytes)?;

    &self.bytes.write_u16::<BigEndian>(0x1234)?;
    &self.bytes.write_u32::<BigEndian>(0x23456789)?;
    &self.bytes.write_u16::<BigEndian>(0xabcd)?;
    &self.bytes.write_u16::<BigEndian>(0xeeee)?;
    &self.bytes.write_u16::<BigEndian>(0xcccc)?;
    &self.bytes.write_u8(0xdd)?;

    chunk.end(&mut self.bytes);
    Ok(())
  }

  pub fn write(memory: &Memory, stack: &Stack) -> Result<Vec<u8>> {
    let mut q = Quetzal { bytes: Vec::new() };
    let chunk = Chunk::start(newId('F', 'O', 'R', 'M'), &mut q.bytes)?;
    q.bytes.write_u32::<BigEndian>(newId('I', 'F', 'Z', 'S'))?;
    q.write_header(memory)?;
    chunk.end(&mut q.bytes);

    Ok(q.bytes)
  }
}

#[cfg(test)]
mod tests {}
