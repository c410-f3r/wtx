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
  fn encode(
    &self,
    fbw: &mut FilledBufferWriter<'_>,
    value: D::EncodeValue<'_>,
  ) -> Result<(), D::Error>;

  /// In rust terms, is the element `Option::None`?
  #[inline]
  fn is_null(&self) -> bool {
    false
  }
}

impl Encode<()> for &str {
  #[inline]
  fn encode(&self, _: &mut FilledBufferWriter<'_>, _: ()) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl<D, T> Encode<D> for &T
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode(
    &self,
    fbw: &mut FilledBufferWriter<'_>,
    value: D::EncodeValue<'_>,
  ) -> Result<(), D::Error> {
    (**self).encode(fbw, value)
  }
}

impl<D, L, R> Encode<D> for Either<L, R>
where
  D: Database,
  L: Encode<D>,
  R: Encode<D>,
{
  #[inline]
  fn encode(
    &self,
    fbw: &mut FilledBufferWriter<'_>,
    value: D::EncodeValue<'_>,
  ) -> Result<(), D::Error> {
    match self {
      Self::Left(left) => left.encode(fbw, value),
      Self::Right(right) => right.encode(fbw, value),
    }
  }
}

impl<D> Encode<D> for &dyn Encode<D>
where
  D: Database,
{
  #[inline]
  fn encode(
    &self,
    fbw: &mut FilledBufferWriter<'_>,
    value: D::EncodeValue<'_>,
  ) -> Result<(), D::Error> {
    (**self).encode(fbw, value)
  }
}

impl<D, T> Encode<D> for Option<T>
where
  D: Database,
  T: Encode<D>,
{
  #[inline]
  fn encode(
    &self,
    fbw: &mut FilledBufferWriter<'_>,
    value: D::EncodeValue<'_>,
  ) -> Result<(), D::Error> {
    match self {
      None => Ok(()),
      Some(elem) => elem.encode(fbw, value),
    }
  }

  #[inline]
  fn is_null(&self) -> bool {
    self.is_none()
  }
}
