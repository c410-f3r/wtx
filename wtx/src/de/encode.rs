use crate::{de::DEController, misc::Either};

/// Encodes itself into a data format.
pub trait Encode<DEC>
where
  DEC: DEController,
{
  /// Performs the conversion.
  fn encode(
    &self,
    aux: &mut DEC::Aux,
    ew: &mut DEC::EncodeWrapper<'_, '_, '_>,
  ) -> Result<(), DEC::Error>;

  /// If this instance can be desired nullable.
  #[inline]
  fn is_null(&self) -> bool {
    false
  }
}

impl Encode<()> for u32 {
  #[inline]
  fn encode(&self, _: &mut (), _: &mut ()) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl Encode<()> for &str {
  #[inline]
  fn encode(&self, _: &mut (), _: &mut ()) -> Result<(), crate::Error> {
    Ok(())
  }
}

impl<DEC, T> Encode<DEC> for &T
where
  DEC: DEController,
  T: Encode<DEC>,
{
  #[inline]
  fn encode(
    &self,
    aux: &mut DEC::Aux,
    ew: &mut DEC::EncodeWrapper<'_, '_, '_>,
  ) -> Result<(), DEC::Error> {
    (**self).encode(aux, ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<DEC, T> Encode<DEC> for &mut T
where
  DEC: DEController,
  T: Encode<DEC>,
{
  #[inline]
  fn encode(
    &self,
    aux: &mut DEC::Aux,
    ew: &mut DEC::EncodeWrapper<'_, '_, '_>,
  ) -> Result<(), DEC::Error> {
    (**self).encode(aux, ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<DEC, L, R> Encode<DEC> for Either<L, R>
where
  DEC: DEController,
  L: Encode<DEC>,
  R: Encode<DEC>,
{
  #[inline]
  fn encode(
    &self,
    aux: &mut DEC::Aux,
    ew: &mut DEC::EncodeWrapper<'_, '_, '_>,
  ) -> Result<(), DEC::Error> {
    match self {
      Self::Left(left) => left.encode(aux, ew),
      Self::Right(right) => right.encode(aux, ew),
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

impl<DEC> Encode<DEC> for &dyn Encode<DEC>
where
  DEC: DEController,
{
  #[inline]
  fn encode(
    &self,
    aux: &mut DEC::Aux,
    ew: &mut DEC::EncodeWrapper<'_, '_, '_>,
  ) -> Result<(), DEC::Error> {
    (**self).encode(aux, ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<DEC, T> Encode<DEC> for Option<T>
where
  DEC: DEController,
  T: Encode<DEC>,
{
  #[inline]
  fn encode(
    &self,
    aux: &mut DEC::Aux,
    ew: &mut DEC::EncodeWrapper<'_, '_, '_>,
  ) -> Result<(), DEC::Error> {
    match self {
      None => Ok(()),
      Some(elem) => elem.encode(aux, ew),
    }
  }

  #[inline]
  fn is_null(&self) -> bool {
    self.is_none()
  }
}
