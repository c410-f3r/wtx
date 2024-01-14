use crate::database::{Database, TableSuffix};
use alloc::string::String;

/// An element that can be represented from one or more database row, in other words, tables
/// with relationships.
pub trait FromRecords<D>: Sized
where
  D: Database,
{
  /// Constructs a single instance based on an arbitrary number of rows.
  fn from_records(
    buffer_cmd: &mut String,
    curr_record: &D::Record<'_>,
    records: &D::Records<'_>,
    table_suffix: TableSuffix,
  ) -> Result<(usize, Self), D::Error>;
}

impl<D> FromRecords<D> for ()
where
  D: Database,
{
  #[inline]
  fn from_records(
    _: &mut String,
    _: &D::Record<'_>,
    _: &D::Records<'_>,
    _: TableSuffix,
  ) -> Result<(usize, Self), D::Error> {
    Ok((0, ()))
  }
}
