use crate::database::{
  Records,
  client::postgres::{Postgres, PostgresCommonRecords, PostgresRecord, PostgresStatement},
};
use core::ops::Range;

/// Records
#[derive(Debug)]
pub struct PostgresRecords<'exec, E> {
  pub(crate) common: PostgresCommonRecords<'exec, E>,
}

impl<'exec, E> PostgresRecords<'exec, E> {
  pub(crate) fn new(
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

  fn get(&self, idx: usize) -> Option<PostgresRecord<'exec, E>> {
    self.common.get(idx)
  }

  fn iter(&self) -> impl Iterator<Item = PostgresRecord<'exec, E>> {
    self.common.iter()
  }

  fn len(&self) -> usize {
    self.common.len()
  }
}

impl<E> Default for PostgresRecords<'_, E> {
  #[inline]
  fn default() -> Self {
    Self::new(&[], &[], PostgresStatement::default(), &[])
  }
}
