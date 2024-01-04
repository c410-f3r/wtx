use crate::{database::Database, misc::FilledBufferWriter};

/// Encodes a type into a byte representation.
pub trait Encode<D>
where
  D: Database,
{
  /// Performs the conversion.
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), D::Error>;

  /// In rust terms, is the element `Option::None`?
  #[inline]
  fn is_null(&self) -> bool {
    false
  }
}

impl<D, T> Encode<D> for &T
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), D::Error> {
    (**self).encode(buffer)
  }
}

impl<D> Encode<D> for &dyn Encode<D>
where
  D: Database,
{
  #[inline]
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), D::Error> {
    (**self).encode(buffer)
  }
}

impl<D, T> Encode<D> for Option<T>
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode(&self, buffer: &mut FilledBufferWriter<'_>) -> Result<(), D::Error> {
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
