use crate::{
  database::{
    Record, ValueIdent,
    client::{
      mysql::{Mysql, MysqlCommonRecord, MysqlStatement},
      rdbms::value,
    },
  },
  misc::{DEController, Lease},
};
use core::ops::Range;

/// Record
#[derive(Debug)]
pub struct MysqlRecord<'exec, E> {
  pub(crate) common: MysqlCommonRecord<'exec, E>,
}

impl<'exec, E> MysqlRecord<'exec, E> {
  #[inline]
  pub(crate) fn new(
    record: &'exec [u8],
    stmt: MysqlStatement<'exec>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { common: MysqlCommonRecord::new(record, stmt, values_params) }
  }
}

impl<'exec, E> Lease<MysqlCommonRecord<'exec, E>> for MysqlRecord<'exec, E> {
  #[inline]
  fn lease(&self) -> &MysqlCommonRecord<'exec, E> {
    &self.common
  }
}

impl<'exec, E> Record<'exec> for MysqlRecord<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn len(&self) -> usize {
    self.common.values_params.len()
  }

  #[inline]
  fn value<CI>(&self, ci: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'exec, '_>>
  where
    CI: ValueIdent<Self>,
  {
    value(ci, self)
  }
}

impl<'exec, E> ValueIdent<MysqlRecord<'exec, E>> for str {
  #[inline]
  fn idx(&self, input: &MysqlRecord<'exec, E>) -> Option<usize> {
    self.idx(&input.common)
  }
}

impl<'exec, E> From<MysqlCommonRecord<'exec, E>> for MysqlRecord<'exec, E> {
  #[inline]
  fn from(from: MysqlCommonRecord<'exec, E>) -> Self {
    Self { common: from }
  }
}
