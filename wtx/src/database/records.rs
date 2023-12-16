use crate::database::Database;

/// A collection of [crate::database::Record].
pub trait Records: Default {
  /// See [Database].
  type Database: Database;

  /// Iterator of records
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'_>>;

  /// The number of records;
  fn len(&self) -> usize;

  /// Tries to retrieve a record.
  fn record(&self, record_idx: usize) -> Option<<Self::Database as Database>::Record<'_>>;
}

impl Records for () {
  type Database = ();

  #[inline]
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'_>> {
    [].into_iter()
  }

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn record(&self, _: usize) -> Option<<Self::Database as Database>::Record<'_>> {
    None
  }
}
