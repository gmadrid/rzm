use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use result::Result;
use zmachine::vm::memory::Memory;
use zmachine::vm::pc::PC;
use zmachine::vm::ptrs::BytePtr;
use zmachine::vm::stack::Stack;

struct Chunk {
  start_offset: usize,
}

fn new_id(str: &str) -> u32 {
  assert!(str.len() == 4);
  let bytes = str.as_bytes();
  BigEndian::read_u32(bytes)
}

impl Chunk {
  fn start(id: u32, bytes: &mut Vec<u8>) -> Result<Chunk> {
    info!(target: "foo", "start: {}", bytes.len());
    bytes.write_u32::<BigEndian>(id)?;
    bytes.write_u32::<BigEndian>(0)?;
    info!(target: "foo", "after: {}", bytes.len());
    Ok(Chunk { start_offset: bytes.len() - 8 })
  }

  fn end(self, bytes: &mut Vec<u8>) -> Result<()> {
    // Subtract 8 to account for the id and size which are not counted in the chunk size.
    let used_bytes = bytes.len() - self.start_offset - 8;
    BigEndian::write_u32(&mut bytes[self.start_offset + 4..], used_bytes as u32);

    if used_bytes % 2 == 1 {
      // Spec says to pad the chunk if odd-length.
      bytes.write_u8(0)?;
    }
    Ok(())
  }
}

pub struct Quetzal {
  bytes: Vec<u8>,
}

impl Quetzal {
  fn write_header(&mut self, memory: &Memory, pc: &PC) -> Result<()> {
    let chunk = Chunk::start(new_id(&"IFhd"), &mut self.bytes)?;

    // TODO: fix magic numbers?
    // release number
    &self.bytes.write_u16::<BigEndian>(memory.u16_at(BytePtr::new(0x02)));

    // serial number
    let ptr = BytePtr::new(0x12);
    for i in 0..6 {
      &self.bytes.write_u8(memory.u8_at(ptr.inc_by(i)));
    }

    // checksum
    &self.bytes.write_u16::<BigEndian>(memory.u16_at(BytePtr::new(0x1c)));

    // PC
    // Awkward, writing 3 bytes of a 4-byte value.
    let pc_as_u32 = usize::from(pc.pc()) as u32;
    &self.bytes.write_u8((pc_as_u32 >> 16) as u8);
    &self.bytes.write_u16::<BigEndian>(pc_as_u32 as u16);

    chunk.end(&mut self.bytes)?;
    Ok(())
  }

  fn write_umem(&mut self, memory: &Memory) -> Result<()> {
    let chunk = Chunk::start(new_id(&"UMem"), &mut self.bytes)?;
    self.bytes.extend_from_slice(memory.dynamic_slice());
    chunk.end(&mut self.bytes)?;
    Ok(())
  }

  pub fn write(memory: &Memory, stack: &Stack, pc: &PC) -> Result<Vec<u8>> {
    let mut q = Quetzal { bytes: Vec::new() };
    let chunk = Chunk::start(new_id(&"FORM"), &mut q.bytes)?;
    q.bytes.write_u32::<BigEndian>(new_id(&"IFZS"))?;
    q.write_header(memory, pc)?;
    q.write_umem(memory)?;
    chunk.end(&mut q.bytes)?;

    Ok(q.bytes)
  }
}

#[cfg(test)]
mod tests {}
