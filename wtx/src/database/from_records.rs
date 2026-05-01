use crate::{
  codec::Decode,
  database::{Database, Records},
  misc::{Lease, into_rslt},
};
use alloc::boxed::Box;
use core::{fmt::Debug, iter};

/// Used by [`FromRecords`].
#[derive(Debug)]
pub struct FromRecordsParams<R> {
  /// The number of records used to construct entities.
  pub consumed_records: usize,
  /// Current field or column index
  pub curr_field_idx: usize,
  /// Current record or row element
  pub curr_record: R,
  /// Current record or row index
  pub curr_record_idx: usize,
  /// If the the current entity is being parsed has a 1:1 relationship.
  pub is_in_one_relationship: bool,
}

impl<'exec, R> FromRecordsParams<R> {
  fn init<D>(records: &D::Records<'exec>) -> Option<Self>
  where
    D: Database<Record<'exec> = R>,
  {
    Some(Self {
      consumed_records: 0,
      curr_field_idx: 0,
      curr_record: records.get(0)?,
      curr_record_idx: 0,
      is_in_one_relationship: false,
    })
  }

  /// Increases the number of consumed records by the given number.
  pub const fn inc_consumed_records(&mut self, value: usize) {
    self.consumed_records = self.consumed_records.wrapping_add(value);
  }

  /// Increases the current field or column index by 1.
  pub const fn inc_field_idx(&mut self) {
    self.curr_field_idx = self.curr_field_idx.wrapping_add(1);
  }

  /// Increases the current record or row index by 1.
  pub const fn inc_record_idx(&mut self) {
    self.curr_record_idx = self.curr_record_idx.wrapping_add(1);
  }
}

/// An element that can be represented from one or more database rows.
pub trait FromRecords<'exec, D>: Sized
where
  D: Database,
{
  /// The number of fields
  const FIELDS: u16;
  /// Field index where the ID is located, if any.
  const ID_IDX: Option<usize>;

  /// The type of the ID.
  ///
  /// `()` can be used for instances without ID.
  type IdTy: Copy + Debug + Decode<'exec, D> + PartialEq;

  /// Used by implementations. Constructs a single instance based on an arbitrary number of rows.
  ///
  /// You must increase [`FromRecordsParams::consumed_records`] based on the number of read
  /// records otherwise [`FromRecords::many`] will short-circuit.
  fn from_records(
    curr_params: &mut FromRecordsParams<D::Record<'exec>>,
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error>;

  /// Used by consumers of this trait. Expects that one or more records can represent zero or more
  /// entities.
  #[inline]
  fn many<R>(records: R) -> impl Iterator<Item = Result<Self, D::Error>>
  where
    R: Lease<D::Records<'exec>>,
  {
    let mut state = FromRecordsParams::init::<D>(records.lease()).map(|el| (el, records));
    iter::from_fn(move || {
      let (params, local_records) = state.as_mut()?;
      let local_records_ref = local_records.lease();
      let record = local_records_ref.get(params.consumed_records)?;
      params.curr_field_idx = 0;
      params.curr_record = record;
      params.curr_record_idx = params.consumed_records;
      let prev_consumed_records = params.consumed_records;
      let rslt = Self::from_records(params, local_records_ref);
      if prev_consumed_records == params.consumed_records {
        return if rslt.is_err() { Some(rslt) } else { None };
      }
      Some(rslt)
    })
  }

  /// Used by consumers of this trait. Expects that one or more records can represent a single
  /// entity.
  #[inline]
  fn single(records: &D::Records<'exec>) -> Result<Self, D::Error> {
    Self::from_records(&mut into_rslt(FromRecordsParams::init::<D>(records))?, records)
  }
}

impl<'exec, D> FromRecords<'exec, D> for ()
where
  D: Database,
  i32: Decode<'exec, D>,
{
  const FIELDS: u16 = 0;
  const ID_IDX: Option<usize> = None;

  type IdTy = i32;

  #[inline]
  fn from_records(
    _: &mut FromRecordsParams<D::Record<'exec>>,
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
  const FIELDS: u16 = T::FIELDS;
  const ID_IDX: Option<usize> = T::ID_IDX;

  type IdTy = T::IdTy;

  #[inline]
  fn from_records(
    curr_params: &mut FromRecordsParams<D::Record<'exec>>,
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error> {
    Ok(Box::new(T::from_records(curr_params, records)?))
  }
}

impl<'exec, D, T> FromRecords<'exec, D> for Option<T>
where
  D: Database,
  T: FromRecords<'exec, D>,
{
  const FIELDS: u16 = T::FIELDS;
  const ID_IDX: Option<usize> = T::ID_IDX;

  type IdTy = T::IdTy;

  #[inline]
  fn from_records(
    curr_params: &mut FromRecordsParams<D::Record<'exec>>,
    records: &D::Records<'exec>,
  ) -> Result<Self, D::Error> {
    if records.len() == 0 { Ok(None) } else { Ok(Some(T::from_records(curr_params, records)?)) }
  }
}
