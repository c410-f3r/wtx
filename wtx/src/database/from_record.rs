use crate::database::Record;

/// An element that can be represented from a single database record. In most cases it means
/// a database table without relationships.
pub trait FromRecord<E, R>: Sized
where
  E: From<crate::Error>,
  R: Record,
{
  /// Fallible entry-point that maps the element.
  fn from_record(record: R) -> Result<Self, E>;
}
