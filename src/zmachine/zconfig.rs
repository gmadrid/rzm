pub trait ZConfig {
  fn stack_size(&self) -> Option<usize> {
    None
  }
}

pub struct ZConfigStack<'a> {
  stack: Vec<&'a ZConfig>,
}

pub struct ZDefaults {}

impl ZDefaults {
  pub fn new() -> ZDefaults {
    ZDefaults {}
  }
}

impl ZConfig for ZDefaults {
  fn stack_size(&self) -> Option<usize> {
    Some(0xf000usize)
  }
}
