use crate::misc::FilledBufferWriter;

/// Encodes a type into a byte representation.
pub trait Encode<C, E>
where
  E: From<crate::Error>,
{
  /// Performs the conversion.
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E>;

  /// In rust terms, is the element `Option::None`?
  #[inline]
  fn is_null(&self) -> bool {
    false
  }
}

impl<C, E, T> Encode<C, E> for &T
where
  E: From<crate::Error>,
  T: Encode<C, E>,
{
  #[inline]
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
    (**self).encode(buffer)
  }
}

impl<C, E> Encode<C, E> for &dyn Encode<C, E>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
    (**self).encode(buffer)
  }
}

impl<C, E, T> Encode<C, E> for Option<T>
where
  E: From<crate::Error>,
  T: Encode<C, E>,
{
  #[inline]
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), E> {
    match self {
      None => Ok(()),
      Some(elem) => elem.encode(buffer),
    }
  }

  #[inline]
  fn is_null(&self) -> bool {
    self.is_none()
  }
}
