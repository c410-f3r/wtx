use crate::database::Database;

/// Any element that has a corresponding database type.
pub trait Typed<D>
where
  D: Database,
{
  /// Concrete type object.
  const TY: Option<D::Ty>;
}

impl<D, T> Typed<D> for &T
where
  D: Database,
  T: Typed<D>,
{
  const TY: Option<D::Ty> = T::TY;
}

impl<D, T> Typed<D> for Option<T>
where
  D: Database,
  T: Typed<D>,
{
  const TY: Option<D::Ty> = T::TY;
}
