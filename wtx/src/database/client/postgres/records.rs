use crate::database::client::postgres::{statements::statement::Statement, Postgres, Record};
use core::{marker::PhantomData, ops::Range};

/// Records
#[derive(Debug)]
pub struct Records<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) phantom: PhantomData<fn() -> E>,
  /// Each element represents a record and an offset of `values_bytes_offsets`.
  pub(crate) records_values_offsets: &'exec [usize],
  pub(crate) stmt: Statement<'exec>,
  /// Each element represents a value and an offset of `bytes`.
  pub(crate) values_bytes_offsets: &'exec [(bool, Range<usize>)],
}

impl<'exec, E> Records<'exec, E> {
  #[inline]
  pub(crate) fn _new(
    bytes: &'exec [u8],
    records_values_offsets: &'exec [usize],
    stmt: Statement<'exec>,
    values_bytes_offsets: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { bytes, phantom: PhantomData, records_values_offsets, stmt, values_bytes_offsets }
  }
}

impl<'exec, E> crate::database::Records<'exec> for Records<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn get(&self, record_idx: usize) -> Option<Record<'exec, E>> {
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
  fn iter(&self) -> impl Iterator<Item = Record<'exec, E>> {
    (0..self.len()).filter_map(|idx| self.get(idx))
  }

  #[inline]
  fn len(&self) -> usize {
    self.records_values_offsets.len()
  }
}

impl<E> Default for Records<'_, E> {
  #[inline]
  fn default() -> Self {
    Self {
      bytes: &[],
      phantom: PhantomData,
      records_values_offsets: &[],
      stmt: Statement::default(),
      values_bytes_offsets: &[],
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::{
      client::postgres::{
        statements::statement::Statement,
        tests::{column0, column1, column2},
        DecodeValue, Record, Records, Ty,
      },
      Record as _, Records as _,
    },
    misc::Vector,
  };

  #[test]
  fn returns_correct_values() {
    let bytes = &[0, 0, 0, 2, 1, 2, 0, 0, 0, 2, 3, 4, 9, 9, 9, 0, 1, 0, 0, 0, 4, 5, 6, 7, 8];
    let values = &[(column0(), Ty::Any), (column1(), Ty::Any), (column2(), Ty::Any)];
    let stmt = Statement::new(3, 0, values);
    let mut records_values_offsets = Vector::new();
    let mut values_bytes_offsets = Vector::new();
    assert_eq!(
      Record::parse(bytes, 0..12, stmt.clone(), &mut values_bytes_offsets, 2).unwrap(),
      Record::<crate::Error>::_new(
        &[1, 2, 0, 0, 0, 2, 3, 4],
        0,
        stmt.clone(),
        &[(false, 0..2), (false, 6..8)]
      )
    );
    records_values_offsets.push(values_bytes_offsets.len()).unwrap();
    assert_eq!(
      Record::parse(bytes, 17..25, stmt.clone(), &mut values_bytes_offsets, 1).unwrap(),
      Record::<crate::Error>::_new(&[5, 6, 7, 8], 17, stmt.clone(), &[(false, 17..21)])
    );
    records_values_offsets.push(values_bytes_offsets.len()).unwrap();

    let records = Records::<crate::Error>::_new(
      &bytes[4..],
      &records_values_offsets,
      stmt.clone(),
      &values_bytes_offsets,
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records.bytes, &bytes[4..]);
    assert_eq!(records.records_values_offsets, &[2, 3]);
    assert_eq!(records.values_bytes_offsets, &[(false, 0..2), (false, 6..8), (false, 17..21)]);

    let first_record = records.get(0).unwrap();
    assert_eq!(
      &first_record,
      &Record::<crate::Error>::_new(
        &[1, 2, 0, 0, 0, 2, 3, 4],
        0,
        stmt.clone(),
        &[(false, 0..2), (false, 6..8)]
      )
    );
    assert_eq!(first_record.value(0).unwrap(), DecodeValue::new(&[1, 2], column0().ty));
    assert_eq!(first_record.value(1).unwrap(), DecodeValue::new(&[3, 4], column1().ty));

    let second_record = records.get(1).unwrap();
    assert_eq!(&second_record, &Record::_new(&[5, 6, 7, 8], 17, stmt.clone(), &[(false, 17..21)]));

    assert_eq!(records.iter().count(), 2);
  }
}
