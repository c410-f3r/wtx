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

/// HTTP/1 header.
pub trait Http1Header: Header {}

impl Http1Header for () {}

impl<T> Http1Header for &T where T: Http1Header {}
