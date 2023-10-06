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

pub trait Http1Header: Header {}

impl Http1Header for () {}

impl<T> Http1Header for &T where T: Http1Header {}
