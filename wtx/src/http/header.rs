pub trait Header {
  fn name(&self) -> &[u8];

  fn value(&self) -> &[u8];
}

impl Header for () {
  #[inline]
  fn name(&self) -> &[u8] {
    &[]
  }

  #[inline]
  fn value(&self) -> &[u8] {
    &[]
  }
}

pub trait Http1Header: Header {}

impl Http1Header for () {}
