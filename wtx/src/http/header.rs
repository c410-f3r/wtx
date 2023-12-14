/// HTTP header.
pub trait Header {
  /// Name.
  fn name(&self) -> &[u8];

  /// Value.
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

impl<T> Header for &T
where
  T: Header,
{
  #[inline]
  fn name(&self) -> &[u8] {
    (*self).name()
  }

  #[inline]
  fn value(&self) -> &[u8] {
    (*self).value()
  }
}

impl Header for [&[u8]; 2] {
  #[inline]
  fn name(&self) -> &[u8] {
    self[0]
  }

  #[inline]
  fn value(&self) -> &[u8] {
    self[1]
  }
}
