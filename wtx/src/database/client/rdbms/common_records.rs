use crate::{
  database::{
    Database, Records,
    client::rdbms::{common_record::CommonRecord, statement::Statement},
  },
  misc::Lease,
};
use core::{marker::PhantomData, ops::Range};

/// Records used by several database implementations
#[derive(Debug)]
pub(crate) struct CommonRecords<'exec, A, C, D, T> {
  pub(crate) phantom: PhantomData<D>,
  pub(crate) records: &'exec [u8],
  /// Each element represents a ***whole*** record. The first element is the number of affected
  /// values, the second element is the range delimitates bytes and the third element if the range
  /// that delimitates `values_params`.
  pub(crate) records_params: &'exec [(Range<usize>, Range<usize>)],
  pub(crate) stmt: Statement<'exec, A, C, T>,
  /// Each element represents the ***data*** of a record that is delimited by the first range of
  /// `records_params`.
  pub(crate) values_params: &'exec [(bool, Range<usize>)],
}

impl<'exec, A, C, D, T> CommonRecords<'exec, A, C, D, T> {
  pub(crate) const fn new(
    records: &'exec [u8],
    records_params: &'exec [(Range<usize>, Range<usize>)],
    stmt: Statement<'exec, A, C, T>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { records, phantom: PhantomData, records_params, stmt, values_params }
  }
}

impl<'exec, A, C, D, T> Records<'exec> for CommonRecords<'exec, A, C, D, T>
where
  A: Clone + Default,
  C: Clone,
  T: Clone,
  D: Database,
  D::Record<'exec>: From<CommonRecord<'exec, A, C, D, T>>,
{
  type Database = D;

  #[inline]
  fn get(&self, idx: usize) -> Option<<Self::Database as Database>::Record<'exec>> {
    do_get::<A, C, D, T>(
      idx,
      self.records_params,
      self.records,
      self.stmt.clone(),
      self.values_params,
    )
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = <Self::Database as Database>::Record<'exec>> {
    (0..self.len()).filter_map(move |idx| {
      do_get::<A, C, D, T>(
        idx,
        self.records_params,
        self.records,
        self.stmt.clone(),
        self.values_params,
      )
    })
  }

  #[inline]
  fn len(&self) -> usize {
    self.records_params.len()
  }
}

impl<'exec, A, C, D, T> Lease<CommonRecords<'exec, A, C, D, T>>
  for CommonRecords<'exec, A, C, D, T>
{
  #[inline]
  fn lease(&self) -> &CommonRecords<'exec, A, C, D, T> {
    self
  }
}

impl<A, C, D, T> Default for CommonRecords<'_, A, C, D, T>
where
  A: Default,
{
  #[inline]
  fn default() -> Self {
    Self {
      records: &[],
      phantom: PhantomData,
      records_params: &[],
      stmt: Statement::default(),
      values_params: &[],
    }
  }
}

#[inline]
fn do_get<'exec, A, C, D, T>(
  idx: usize,
  records_params: &'exec [(Range<usize>, Range<usize>)],
  records: &'exec [u8],
  stmt: Statement<'exec, A, C, T>,
  values_params: &'exec [(bool, Range<usize>)],
) -> Option<<D as Database>::Record<'exec>>
where
  D: Database,
  D::Record<'exec>: From<CommonRecord<'exec, A, C, D, T>>,
{
  let (record_range, values_range) = records_params.get(idx)?;
  let common_record = CommonRecord::new(
    records.get(record_range.start..record_range.end)?,
    stmt,
    values_params.get(values_range.start..values_range.end)?,
  );
  Some(common_record.into())
}
