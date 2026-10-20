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
    affected_rows: u64,
    records: &'exec [u8],
    records_params: &'exec [(Range<usize>, Range<usize>)],
    stmt: PostgresStatement<'exec>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self {
      common: PostgresCommonRecords::new(
        affected_rows,
        records,
        records_params,
        stmt,
        values_params,
      ),
    }
  }
}

impl<'exec, E> Records<'exec> for PostgresRecords<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn affected_rows(&self) -> u64 {
    self.common.affected_rows()
  }

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
    Self::new(0, &[], &[], PostgresStatement::default(), &[])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn affected_rows_support_the_full_protocol_range() {
    let affected_rows = u64::from(u32::MAX);
    let records = PostgresRecords::<crate::Error>::new(
      affected_rows,
      &[],
      &[],
      PostgresStatement::default(),
      &[],
    );
    assert_eq!(records.affected_rows(), affected_rows);
  }
}
