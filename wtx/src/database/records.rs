use crate::{database::Database, misc::Lease};

/// A collection of [`crate::database::Record`].
pub trait Records<'client>: Default + Lease<Self> {
  /// See [Database].
  type Database: Database;

  /// The number of rows affected by the executed command.
  fn affected_rows(&self) -> u64;

  /// Tries to retrieve a record.
  fn get(&self, record_idx: usize) -> Option<<Self::Database as Database>::Record<'client>>;

  /// Iterator of records.
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'client>>;

  /// The number of records.
  fn len(&self) -> usize;
}

impl<'client> Records<'client> for () {
  type Database = ();

  #[inline]
  fn affected_rows(&self) -> u64 {
    0
  }

  #[inline]
  fn get(&self, _: usize) -> Option<<Self::Database as Database>::Record<'client>> {
    None
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'client>> {
    [].into_iter()
  }

  #[inline]
  fn len(&self) -> usize {
    0
  }
}
