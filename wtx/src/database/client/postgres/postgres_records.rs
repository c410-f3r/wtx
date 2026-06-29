use crate::{
  database::{
    Records,
    client::postgres::{Postgres, PostgresCommonRecords, PostgresRecord, PostgresStatement},
  },
  misc::Lease,
};
use core::ops::Range;

/// Records
#[derive(Debug)]
pub struct PostgresRecords<'exec, E> {
  pub(crate) common: PostgresCommonRecords<'exec, E>,
}

impl<'exec, E> PostgresRecords<'exec, E> {
  pub(crate) const fn new(
    records: &'exec [u8],
    records_params: &'exec [(Range<usize>, Range<usize>)],
    stmt: PostgresStatement<'exec>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { common: PostgresCommonRecords::new(records, records_params, stmt, values_params) }
  }
}

impl<'exec, E> Records<'exec> for PostgresRecords<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn get(&self, record_idx: usize) -> Option<PostgresRecord<'exec, E>> {
    self.common.get(record_idx)
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = PostgresRecord<'exec, E>> {
    self.common.iter()
  }

  #[inline]
  fn len(&self) -> usize {
    self.common.len()
  }
}

impl<'exec, E> Lease<PostgresRecords<'exec, E>> for PostgresRecords<'exec, E> {
  #[inline]
  fn lease(&self) -> &PostgresRecords<'exec, E> {
    self
  }
}

impl<E> Default for PostgresRecords<'_, E> {
  #[inline]
  fn default() -> Self {
    Self::new(&[], &[], PostgresStatement::default(), &[])
  }
}
