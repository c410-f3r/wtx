use crate::{
  database::Database,
  misc::{Either, FilledBufferWriter},
};

/// Encodes a type into a byte representation.
pub trait Encode<D>
where
  D: Database,
{
  /// Performs the conversion.
  fn encode(&self, ev: &mut D::EncodeValue<'_, '_>) -> Result<(), D::Error>;

  /// In rust terms, is the element `Option::None`?
  #[inline]
  fn is_null(&self) -> bool {
    false
  }
}

impl Encode<()> for u32 {
  #[inline]
  fn encode(&self, _: &mut FilledBufferWriter<'_>) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl Encode<()> for &str {
  #[inline]
  fn encode(&self, _: &mut FilledBufferWriter<'_>) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl<D, T> Encode<D> for &T
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode(&self, ev: &mut D::EncodeValue<'_, '_>) -> Result<(), D::Error> {
    (**self).encode(ev)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<D, L, R> Encode<D> for Either<L, R>
where
  D: Database,
  L: Encode<D>,
  R: Encode<D>,
{
  #[inline]
  fn encode(&self, ev: &mut D::EncodeValue<'_, '_>) -> Result<(), D::Error> {
    match self {
      Self::Left(left) => left.encode(ev),
      Self::Right(right) => right.encode(ev),
    }
  }

  #[inline]
  fn is_null(&self) -> bool {
    match self {
      Self::Left(left) => left.is_null(),
      Self::Right(right) => right.is_null(),
    }
  }
}

impl<D> Encode<D> for &dyn Encode<D>
where
  D: Database,
{
  #[inline]
  fn encode(&self, ev: &mut D::EncodeValue<'_, '_>) -> Result<(), D::Error> {
    (**self).encode(ev)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<D, T> Encode<D> for Option<T>
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode(&self, ev: &mut D::EncodeValue<'_, '_>) -> Result<(), D::Error> {
    match self {
      None => Ok(()),
      Some(elem) => elem.encode(ev),
    }
  }

  #[inline]
  fn is_null(&self) -> bool {
    self.is_none()
  }
}
