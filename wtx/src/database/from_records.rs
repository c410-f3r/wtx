use crate::database::{Database, TableSuffix};
use alloc::{boxed::Box, string::String};

/// An element that can be represented from one or more database row, in other words, tables
/// with relationships.
pub trait FromRecords<'exec, D>: Sized
where
  D: Database,
{
  /// Constructs a single instance based on an arbitrary number of rows.
  fn from_records(
    buffer_cmd: &mut String,
    curr_record: &D::Record<'exec>,
    records: &D::Records<'exec>,
    table_suffix: TableSuffix,
  ) -> Result<(usize, Self), D::Error>;
}

impl<'exec, D> FromRecords<'exec, D> for ()
where
  D: Database,
{
  #[inline]
  fn from_records(
    _: &mut String,
    _: &D::Record<'exec>,
    _: &D::Records<'exec>,
    _: TableSuffix,
  ) -> Result<(usize, Self), D::Error> {
    Ok((0, ()))
  }
}

impl<'exec, D, T> FromRecords<'exec, D> for Box<T>
where
  D: Database,
  T: FromRecords<'exec, D>,
{
  #[inline]
  fn from_records(
    buffer_cmd: &mut String,
    curr_record: &D::Record<'exec>,
    records: &D::Records<'exec>,
    table_suffix: TableSuffix,
  ) -> Result<(usize, Self), D::Error> {
    let (n, this) = T::from_records(buffer_cmd, curr_record, records, table_suffix)?;
    Ok((n, Box::new(this)))
  }
}
