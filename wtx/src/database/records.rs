use crate::database::Database;

/// A collection of [crate::database::Record].
pub trait Records: Default {
  /// See [Database].
  type Database: Database;

  /// The number of records;
  fn len(&self) -> usize;

  /// Tries to retrieve a record.
  fn record(&self, record_idx: usize) -> Option<<Self::Database as Database>::Record<'_>>;
}

impl Records for () {
  type Database = ();

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn record(&self, _: usize) -> Option<<Self::Database as Database>::Record<'_>> {
    None
  }
}
