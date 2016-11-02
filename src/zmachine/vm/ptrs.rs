/**
 * The ZMachine uses three types of pointers:
 *   BytePtr: an offset from the base of memory.
 *   WordPtr: an even address in the bottom 128K of memory. Equal to
 *     BytePtr * 2 of the address. (Used only by abbrev. table.)
 *   PackedAddr: specifies where a routine or string begins in high memory.
 * All of these will fit into a 16-bit word.
 */
/**
 * An offset into memory from 0...last byte of static memory.
 */
#[derive(Clone,Copy,Debug)]
pub struct BytePtr {
  val: u16,
}

impl BytePtr {
  fn new(val: u16) -> BytePtr {
    BytePtr { val: val }
  }
}

/**
 * An even address in the bottom 128K of memory. Represented in zcode by
 * the address divided by 2. Used only by the abbrev. table.
 */
#[derive(Clone,Copy,Debug)]
pub struct WordPtr {
  val: u16,
}

impl WordPtr {
  fn new(val: u16) -> WordPtr {
    WordPtr { val: val }
  }
}

/**
 * Specifies the location of a routine or string in high memory.
 * Interpreted differently on every version of the ZMachine.
 */
#[derive(Clone,Copy,Debug)]
pub struct PackedAddr {
  val: u16,
}

impl PackedAddr {
  fn new(val: u16) -> PackedAddr {
    PackedAddr { val: val }
  }
}

/**
 * A pointer into memory. Can refer to any location in memory: static, dynamic,
 * or high memory.
 * Used by the Memory module to refer to bytes in memory.
 */
#[derive(Clone,Copy,Debug)]
pub struct RawPtr {
  val: usize,
}

impl From<RawPtr> for usize {
  fn from(rp: RawPtr) -> usize {
    rp.val
  }
}

impl From<BytePtr> for RawPtr {
  // TODO: perhaps we can check the range of the byteptr?
  fn from(bp: BytePtr) -> RawPtr {
    RawPtr { val: bp.val as usize }
  }
}

impl From<WordPtr> for RawPtr {
  fn from(wp: WordPtr) -> RawPtr {
    RawPtr { val: wp.val as usize * 2 }
  }
}

impl From<PackedAddr> for RawPtr {
  fn from(pa: PackedAddr) -> RawPtr {
    // The v3 behavior.
    RawPtr { val: pa.val as usize * 2 }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_bp_convert() {
    let bp = BytePtr::new(10);
    let rp: RawPtr = bp.into();
    assert_eq!(10usize, rp.into());
  }

  #[test]
  fn test_wp_convert() {
    let wp = WordPtr::new(25);
    let rp: RawPtr = wp.into();
    assert_eq!(50usize, rp.into());
  }

  #[test]
  fn test_pa_convert() {
    let pa = PackedAddr::new(83);
    let rp: RawPtr = pa.into();
    assert_eq!(166usize, rp.into());
  }
}
