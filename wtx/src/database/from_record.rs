use crate::database::Database;

/// An element that can be represented from a single database record. In most cases it means
/// a database table without relationships.
pub trait FromRecord<D>: Sized
where
  D: Database,
{
  /// Fallible entry-point that maps the element.
  fn from_record(record: &D::Record<'_>) -> Result<Self, D::Error>;
}

impl<D> FromRecord<D> for ()
where
  D: Database,
{
  #[inline]
  fn from_record(_: &D::Record<'_>) -> Result<Self, D::Error> {
    Ok(())
  }
}
