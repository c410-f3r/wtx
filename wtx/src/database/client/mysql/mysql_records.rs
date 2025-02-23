use crate::database::{
  Records,
  client::mysql::{Mysql, MysqlRecord, MysqlStatement},
};
use core::{marker::PhantomData, ops::Range};

/// Records
#[derive(Debug)]
pub struct MysqlRecords<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) phantom: PhantomData<fn() -> E>,
  /// Each element represents a record and an offset of `values_bytes_offsets`.
  pub(crate) records_values_offsets: &'exec [usize],
  pub(crate) stmt: MysqlStatement<'exec>,
  /// Each element represents a value and an offset of `bytes`.
  pub(crate) values_bytes_offsets: &'exec [(bool, Range<usize>)],
}

impl<'exec, E> MysqlRecords<'exec, E> {
  #[inline]
  pub(crate) fn new(
    bytes: &'exec [u8],
    records_values_offsets: &'exec [usize],
    stmt: MysqlStatement<'exec>,
    values_bytes_offsets: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { bytes, phantom: PhantomData, records_values_offsets, stmt, values_bytes_offsets }
  }
}

impl<'exec, E> Records<'exec> for MysqlRecords<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn get(&self, record_idx: usize) -> Option<MysqlRecord<'exec, E>> {
    let slice = self.records_values_offsets.get(..record_idx.wrapping_add(1))?;
    let (record_bytes_range, record_values_bytes_offsets) = match slice {
      [] => return None,
      &[to] => {
        let record_values_bytes_offsets = self.values_bytes_offsets.get(..to)?;
        (0..record_values_bytes_offsets.last()?.1.end, record_values_bytes_offsets)
      }
      &[.., from, to] => {
        let record_values_bytes_offsets = self.values_bytes_offsets.get(from..to)?;
        let [start, end] = match record_values_bytes_offsets {
          [(_, first)] => [first.start, first.end],
          [(_, first), .., last] => [first.start, last.1.end],
          _ => return None,
        };
        (start..end, record_values_bytes_offsets)
      }
    };
    Some(MysqlRecord::new(
      self.bytes.get(record_bytes_range)?,
      self.stmt.clone(),
      record_values_bytes_offsets,
    ))
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = MysqlRecord<'exec, E>> {
    (0..self.len()).filter_map(|idx| self.get(idx))
  }

  #[inline]
  fn len(&self) -> usize {
    self.records_values_offsets.len()
  }
}

impl<E> Default for MysqlRecords<'_, E> {
  #[inline]
  fn default() -> Self {
    Self {
      bytes: &[],
      phantom: PhantomData,
      records_values_offsets: &[],
      stmt: MysqlStatement::default(),
      values_bytes_offsets: &[],
    }
  }
}
