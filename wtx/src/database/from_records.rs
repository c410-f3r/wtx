use crate::{
  database::{Database, Decode, Records},
  misc::into_rslt,
};
use alloc::boxed::Box;

/// An element that can be represented from one or more database rows, in other words, entities
/// with relationships.
pub trait FromRecords<'exec, D>: Sized
where
  D: Database,
{
  /// Constructs a single instance based on an arbitrary number of rows.
  fn from_records(
    col_idx: &mut usize,
    curr_record: &D::Record<'exec>,
    curr_record_idx: &mut usize,
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error>;

  /// Should be called once in the initialization phase.
  #[inline]
  fn from_records_initial(records: &D::Records<'exec>) -> Result<Self, D::Error>
  where
    for<'de> u64: Decode<'de, D>,
  {
    Self::from_records(&mut 0, &into_rslt(records.get(0))?, &mut 0, records)
  }
}

impl<'exec, D> FromRecords<'exec, D> for ()
where
  D: Database,
{
  #[inline]
  fn from_records(
    _: &mut usize,
    _: &D::Record<'exec>,
    _: &mut usize,
    _: &D::Records<'exec>,
  ) -> Result<Self, D::Error> {
    Ok(())
  }
}

impl<'exec, D, T> FromRecords<'exec, D> for Box<T>
where
  D: Database,
  T: FromRecords<'exec, D>,
{
  #[inline]
  fn from_records(
    col_idx: &mut usize,
    curr_record: &D::Record<'exec>,
    curr_record_idx: &mut usize,
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error> {
    Ok(Box::new(T::from_records(col_idx, curr_record, curr_record_idx, records)?))
  }
}
