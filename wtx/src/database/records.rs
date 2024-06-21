use crate::database::Database;

/// A collection of [`crate::database::Record`].
pub trait Records<'exec>: Default {
  /// See [Database].
  type Database: Database;

  /// Tries to retrieve a record.
  fn get(&self, record_idx: usize) -> Option<<Self::Database as Database>::Record<'exec>>;

  /// Iterator of records.
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'exec>>;

  /// The number of records.
  fn len(&self) -> usize;
}

impl<'exec> Records<'exec> for () {
  type Database = ();

  #[inline]
  fn get(&self, _: usize) -> Option<<Self::Database as Database>::Record<'exec>> {
    None
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'exec>> {
    [].into_iter()
  }

  #[inline]
  fn len(&self) -> usize {
    0
  }
}
