use crate::database::client::postgres::{statements::Statement, Postgres, Record};
use core::{marker::PhantomData, ops::Range};

/// Records
#[derive(Debug)]
pub struct Records<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) phantom: PhantomData<E>,
  /// Each element represents a record and an offset of `values_bytes_offsets`.
  pub(crate) records_values_offsets: &'exec [usize],
  pub(crate) stmt: Statement<'exec>,
  /// Each element represents a value and an offset of `bytes`.
  pub(crate) values_bytes_offsets: &'exec [(bool, Range<usize>)],
}

impl<'exec, E> crate::database::Records for Records<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn get(&self, record_idx: usize) -> Option<Record<'_, E>> {
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
    let initial_value_offset = record_bytes_range.start;
    Some(Record {
      bytes: self.bytes.get(record_bytes_range)?,
      initial_value_offset,
      stmt: self.stmt.clone(),
      values_bytes_offsets: record_values_bytes_offsets,
      phantom: PhantomData,
    })
  }

  #[inline]
  fn iter(&self) -> impl Iterator<Item = Record<'_, E>> {
    (0..self.len()).filter_map(|idx| self.get(idx))
  }

  #[inline]
  fn len(&self) -> usize {
    self.records_values_offsets.len()
  }
}

impl<'exec, E> Default for Records<'exec, E> {
  #[inline]
  fn default() -> Self {
    Self {
      bytes: <_>::default(),
      phantom: PhantomData,
      records_values_offsets: <_>::default(),
      stmt: <_>::default(),
      values_bytes_offsets: <_>::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use core::marker::PhantomData;

  use crate::database::{
    client::postgres::{statements::Statement, Record, Records},
    Record as _, Records as _,
  };
  use alloc::vec::Vec;

  #[test]
  fn returns_correct_values() {
    let bytes = &[0, 0, 0, 2, 1, 2, 0, 0, 0, 2, 3, 4, 9, 9, 9, 0, 1, 0, 0, 0, 4, 5, 6, 7, 8];
    let mut records_values_offsets = Vec::new();
    let mut values_bytes_offsets = Vec::new();
    assert_eq!(
      Record::parse(bytes, 0..12, Statement::new(&[], &[]), &mut values_bytes_offsets, 2).unwrap(),
      Record {
        bytes: &[1, 2, 0, 0, 0, 2, 3, 4],
        initial_value_offset: 0,
        phantom: PhantomData::<crate::Error>,
        stmt: Statement::new(&[], &[]),
        values_bytes_offsets: &[(false, 0..2), (false, 6..8)]
      }
    );
    records_values_offsets.push(values_bytes_offsets.len());
    assert_eq!(
      Record::parse(bytes, 17..25, Statement::new(&[], &[]), &mut values_bytes_offsets, 1).unwrap(),
      Record {
        bytes: &[5, 6, 7, 8],
        initial_value_offset: 17,
        phantom: PhantomData::<crate::Error>,
        stmt: Statement::new(&[], &[]),
        values_bytes_offsets: &[(false, 17..21)]
      }
    );
    records_values_offsets.push(values_bytes_offsets.len());

    let records = Records {
      bytes: &bytes[4..],
      phantom: PhantomData::<crate::Error>,
      records_values_offsets: &records_values_offsets,
      stmt: Statement::new(&[], &[]),
      values_bytes_offsets: &values_bytes_offsets,
    };
    assert_eq!(records.len(), 2);
    assert_eq!(records.bytes, &bytes[4..]);
    assert_eq!(records.records_values_offsets, &[2, 3]);
    assert_eq!(records.values_bytes_offsets, &[(false, 0..2), (false, 6..8), (false, 17..21)]);

    let first_record = records.get(0).unwrap();
    assert_eq!(
      &first_record,
      &Record {
        bytes: &[1, 2, 0, 0, 0, 2, 3, 4],
        initial_value_offset: 0,
        phantom: PhantomData::<crate::Error>,
        stmt: Statement::new(&[], &[]),
        values_bytes_offsets: &[(false, 0..2), (false, 6..8)]
      }
    );
    assert_eq!(first_record.value(0).unwrap().bytes(), &[1, 2]);
    assert_eq!(first_record.value(1).unwrap().bytes(), &[3, 4]);

    let second_record = records.get(1).unwrap();
    assert_eq!(
      &second_record,
      &Record {
        bytes: &[5, 6, 7, 8],
        initial_value_offset: 17,
        phantom: PhantomData::<crate::Error>,
        stmt: Statement::new(&[], &[]),
        values_bytes_offsets: &[(false, 17..21)]
      }
    );

    assert_eq!(records.iter().count(), 2);
  }
}
