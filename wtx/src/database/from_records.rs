use crate::database::{Database, TableSuffix};
use alloc::string::String;

/// An element that can be represented from one or more database row, in other words, tables
/// with relationships.
pub trait FromRecords: Sized {
  /// See [Database].
  type Database: Database;
  /// Error.
  type Error: From<crate::Error>;

  /// Constructs a single instance based on an arbitrary number of rows.
  fn from_records(
    buffer_cmd: &mut String,
    curr_record: &<Self::Database as Database>::Record<'_>,
    records: &<Self::Database as Database>::Records<'_>,
    table_suffix: TableSuffix,
  ) -> Result<(usize, Self), Self::Error>;
}
