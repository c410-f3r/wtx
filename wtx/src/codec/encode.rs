use crate::{codec::CodecController, misc::Either};

/// Encodes itself into a data format.
pub trait Encode<CC>
where
  CC: CodecController,
{
  /// Performs the conversion.
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error>;

  /// If this instance can be desired nullable.
  #[inline]
  fn is_null(&self) -> bool {
    false
  }
}

impl Encode<()> for u32 {
  #[inline]
  fn encode(&self, _: &mut ()) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl Encode<()> for &str {
  #[inline]
  fn encode(&self, _: &mut ()) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl<CC, T> Encode<CC> for &T
where
  CC: CodecController,
  T: Encode<CC>,
{
  #[inline]
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error> {
    (**self).encode(ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<CC, T> Encode<CC> for &mut T
where
  CC: CodecController,
  T: Encode<CC>,
{
  #[inline]
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error> {
    (**self).encode(ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<CC, L, R> Encode<CC> for Either<L, R>
where
  CC: CodecController,
  L: Encode<CC>,
  R: Encode<CC>,
{
  #[inline]
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error> {
    match self {
      Self::Left(left) => left.encode(ew),
      Self::Right(right) => right.encode(ew),
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

impl<CC> Encode<CC> for &dyn Encode<CC>
where
  CC: CodecController,
{
  #[inline]
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error> {
    (**self).encode(ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<CC, T> Encode<CC> for Option<T>
where
  CC: CodecController,
  T: Encode<CC>,
{
  #[inline]
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error> {
    match self {
      None => Ok(()),
      Some(elem) => elem.encode(ew),
    }
  }

  #[inline]
  fn is_null(&self) -> bool {
    self.is_none()
  }
}
