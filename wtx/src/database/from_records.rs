use crate::{
  database::{Database, Records},
  misc::{Decode, into_rslt},
};
use alloc::boxed::Box;
use core::fmt::Debug;

/// An element that can be represented from one or more database rows.
pub trait FromRecords<'exec, D>: Sized
where
  D: Database,
{
  /// Where the ID is located, if any.
  const ID_IDX: Option<usize>;

  /// The type of the ID.
  ///
  /// `()` can be used for instances without ID.
  type IdTy: Copy + Debug + Decode<'exec, D> + PartialEq;

  /// Constructs a single instance based on an arbitrary number of rows.
  fn from_records(
    curr_params: (&mut usize, &D::Record<'exec>, &mut usize),
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error>;

  /// Should be called once in the initialization phase.
  #[inline]
  fn from_records_initial(records: &D::Records<'exec>) -> Result<Self, D::Error> {
    let curr_field_idx = &mut 0;
    let curr_record = &into_rslt(records.get(0))?;
    let curr_record_idx = &mut 0;
    Self::from_records((curr_field_idx, curr_record, curr_record_idx), records)
  }
}

impl<'exec, D, T> FromRecords<'exec, D> for Box<T>
where
  D: Database,
  T: FromRecords<'exec, D>,
{
  const ID_IDX: Option<usize> = T::ID_IDX;

  type IdTy = T::IdTy;

  #[inline]
  fn from_records(
    (curr_field_idx, curr_record, curr_record_idx): (&mut usize, &D::Record<'exec>, &mut usize),
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error> {
    Ok(Box::new(T::from_records((curr_field_idx, curr_record, curr_record_idx), records)?))
  }
}
