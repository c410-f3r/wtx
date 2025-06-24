use crate::{database::Database, de::Encode, misc::Either};

/// Any element that has a corresponding database type.
pub trait Typed<D>
where
  D: Database,
{
  /// Type that is only known at runtime.
  fn runtime_ty(&self) -> Option<D::Ty>;

  /// Type that is known at compile time.
  fn static_ty() -> Option<D::Ty>
  where
    Self: Sized;
}

impl<D, T> Typed<D> for &T
where
  D: Database,
  T: Typed<D>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<D::Ty> {
    (**self).runtime_ty()
  }

  #[inline]
  fn static_ty() -> Option<D::Ty> {
    T::static_ty()
  }
}

impl<D> Typed<D> for &dyn Typed<D>
where
  D: Database,
{
  #[inline]
  fn runtime_ty(&self) -> Option<D::Ty> {
    (**self).runtime_ty()
  }

  #[inline]
  fn static_ty() -> Option<D::Ty> {
    None
  }
}

impl<D, L, R> Typed<D> for Either<L, R>
where
  D: Database,
  L: Typed<D>,
  R: Typed<D>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<D::Ty> {
    match self {
      Either::Left(elem) => elem.runtime_ty(),
      Either::Right(elem) => elem.runtime_ty(),
    }
  }

  #[inline]
  fn static_ty() -> Option<D::Ty> {
    None
  }
}

impl<D, T> Typed<D> for Option<T>
where
  D: Database,
  T: Typed<D>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<D::Ty> {
    T::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<D::Ty> {
    T::static_ty()
  }
}

/// Marker used for elements that implement both [`Encode`] and [`Typed`].
pub trait TypedEncode<D>: Encode<D> + Typed<D>
where
  D: Database,
{
}

impl<D> Encode<D> for &dyn TypedEncode<D>
where
  D: Database,
{
  #[inline]
  fn encode(&self, aux: &mut D::Aux, ew: &mut D::EncodeWrapper<'_, '_>) -> Result<(), D::Error> {
    (**self).encode(aux, ew)
  }

  #[inline]
  fn is_null(&self) -> bool {
    (**self).is_null()
  }
}

impl<D> Typed<D> for &dyn TypedEncode<D>
where
  D: Database,
{
  #[inline]
  fn runtime_ty(&self) -> Option<D::Ty> {
    (**self).runtime_ty()
  }

  #[inline]
  fn static_ty() -> Option<D::Ty> {
    None
  }
}

impl<D, T> TypedEncode<D> for T
where
  D: Database,
  T: Encode<D> + Typed<D>,
{
}
