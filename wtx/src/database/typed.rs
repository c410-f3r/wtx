use crate::database::Database;

/// Any element that has a corresponding database type.
pub trait Typed<D>
where
  D: Database,
{
  /// Concrete type object.
  const TY: D::Ty;
}

impl<D, T> Typed<D> for &T
where
  D: Database,
  T: Typed<D>,
{
  const TY: D::Ty = T::TY;
}
