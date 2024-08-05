/// HTTP header.
pub trait GenericHeader {
  /// Name.
  fn name(&self) -> &[u8];

  /// Value.
  fn value(&self) -> &[u8];
}

impl GenericHeader for () {
  #[inline]
  fn name(&self) -> &[u8] {
    &[]
  }

  #[inline]
  fn value(&self) -> &[u8] {
    &[]
  }
}

impl<T> GenericHeader for &T
where
  T: GenericHeader,
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

impl GenericHeader for [&[u8]; 2] {
  #[inline]
  fn name(&self) -> &[u8] {
    self[0]
  }

  #[inline]
  fn value(&self) -> &[u8] {
    self[1]
  }
}

#[cfg(feature = "httparse")]
mod httparse {
  use crate::http::GenericHeader;

  impl GenericHeader for httparse::Header<'_> {
    #[inline]
    fn name(&self) -> &[u8] {
      self.name.as_bytes()
    }

    #[inline]
    fn value(&self) -> &[u8] {
      self.value
    }
  }
}
