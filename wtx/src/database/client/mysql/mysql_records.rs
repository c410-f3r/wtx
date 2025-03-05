use crate::database::{
  Records,
  client::mysql::{Mysql, MysqlCommonRecords, MysqlRecord, MysqlStatement},
};
use core::ops::Range;

/// Records
#[derive(Debug)]
pub struct MysqlRecords<'exec, E> {
  pub(crate) common: MysqlCommonRecords<'exec, E>,
}

impl<'exec, E> MysqlRecords<'exec, E> {
  #[inline]
  pub(crate) fn new(
    records: &'exec [u8],
    records_params: &'exec [(Range<usize>, Range<usize>)],
    stmt: MysqlStatement<'exec>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { common: MysqlCommonRecords::new(records, records_params, stmt, values_params) }
  }
}

impl<'exec, E> Records<'exec> for MysqlRecords<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn get(&self, idx: usize) -> Option<MysqlRecord<'exec, E>> {
    self.common.get(idx)
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = MysqlRecord<'exec, E>> {
    self.common.iter()
  }

  #[inline]
  fn len(&self) -> usize {
    self.common.len()
  }
}

impl<E> Default for MysqlRecords<'_, E> {
  #[inline]
  fn default() -> Self {
    Self::new(&[], &[], MysqlStatement::default(), &[])
  }
}
