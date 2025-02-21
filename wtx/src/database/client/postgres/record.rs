use crate::{
  database::{
    client::postgres::{statements::statement::Statement, DecodeValue, Postgres, PostgresError},
    Database, ValueIdent,
  },
  misc::{Vector, _unlikely_dflt, _unlikely_elem},
};
use core::{marker::PhantomData, ops::Range};

/// Record
#[derive(Debug)]
pub struct Record<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) initial_value_offset: usize,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stmt: Statement<'exec>,
  pub(crate) values_bytes_offsets: &'exec [(bool, Range<usize>)],
}

impl<'exec, E> Record<'exec, E> {
  #[inline]
  pub(crate) fn _new(
    bytes: &'exec [u8],
    initial_value_offset: usize,
    stmt: Statement<'exec>,
    values_bytes_offsets: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { bytes, initial_value_offset, phantom: PhantomData, stmt, values_bytes_offsets }
  }

  pub(crate) fn parse(
    mut bytes: &'exec [u8],
    bytes_range: Range<usize>,
    stmt: Statement<'exec>,
    values_bytes_offsets: &'exec mut Vector<(bool, Range<usize>)>,
    values_len: u16,
  ) -> crate::Result<Self> {
    let values_bytes_offsets_start = values_bytes_offsets.len();

    let mut fun = |curr_value_offset: &mut usize, [a, b, c, d]: [u8; 4]| {
      let begin = *curr_value_offset;
      let n = i32::from_be_bytes([a, b, c, d]);
      let (is_null, end) = match n {
        -1 => (true, begin),
        _ => (false, begin.wrapping_add(usize::try_from(n)?)),
      };
      values_bytes_offsets.push((is_null, begin..end))?;
      *curr_value_offset = end;
      crate::Result::Ok(())
    };

    let initial_value_offset = bytes_range.start;
    let mut curr_value_offset = bytes_range.start;

    let local_bytes = bytes.get(bytes_range);
    match (local_bytes, values_len) {
      (Some([a, b, c, d, rest @ ..]), 1..=u16::MAX) => {
        bytes = rest;
        fun(&mut curr_value_offset, [*a, *b, *c, *d])?;
      }
      _ => {
        return Ok(Self {
          bytes,
          initial_value_offset: 0,
          phantom: PhantomData,
          stmt,
          values_bytes_offsets,
        });
      }
    }

    for _ in 1..values_len {
      let idx = curr_value_offset.wrapping_sub(initial_value_offset);
      let Some(&[a, b, c, d, ..]) = bytes.get(idx..) else {
        return Err(PostgresError::InvalidPostgresRecord.into());
      };
      curr_value_offset = curr_value_offset.wrapping_add(4);
      fun(&mut curr_value_offset, [a, b, c, d])?;
    }

    Ok(Self {
      bytes,
      initial_value_offset,
      phantom: PhantomData,
      stmt,
      values_bytes_offsets: values_bytes_offsets
        .get(values_bytes_offsets_start..)
        .unwrap_or_else(_unlikely_dflt),
    })
  }
}

impl<'exec, E> crate::database::Record<'exec> for Record<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn len(&self) -> usize {
    self.values_bytes_offsets.len()
  }

  #[inline]
  fn value<CI>(&self, ci: CI) -> Option<<Self::Database as Database>::DecodeValue<'exec>>
  where
    CI: ValueIdent<Self>,
  {
    let idx = ci.idx(self)?;
    let (is_null, range) = match self.values_bytes_offsets.get(idx) {
      None => return _unlikely_elem(None),
      Some(elem) => elem,
    };
    if *is_null {
      None
    } else {
      let begin = range.start.wrapping_sub(self.initial_value_offset);
      let column = match self.stmt.column(idx) {
        None => return _unlikely_elem(None),
        Some(elem) => elem,
      };
      let end = range.end.wrapping_sub(self.initial_value_offset);
      let bytes = match self.bytes.get(begin..end) {
        None => return _unlikely_elem(None),
        Some(elem) => elem,
      };
      Some(DecodeValue::new(bytes, column.ty))
    }
  }
}

impl<'exec, E> ValueIdent<Record<'exec, E>> for str {
  #[inline]
  fn idx(&self, input: &Record<'exec, E>) -> Option<usize> {
    input.stmt.columns().position(|column| column.name.as_str() == self)
  }
}

impl<E> PartialEq for Record<'_, E> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.bytes == other.bytes
      && self.initial_value_offset == other.initial_value_offset
      && self.phantom == other.phantom
      && self.stmt == other.stmt
      && self.values_bytes_offsets == other.values_bytes_offsets
  }
}

mod array {
  use crate::{
    database::{client::postgres::Postgres, FromRecord, Record},
    misc::{from_utf8_basic, into_rslt, ArrayString},
  };

  impl<E, const N: usize> FromRecord<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn from_record(
      record: &crate::database::client::postgres::record::Record<'_, E>,
    ) -> Result<Self, E> {
      Ok(from_utf8_basic(into_rslt(record.value(0))?.bytes()).map_err(From::from)?.try_into()?)
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
        DecodeValue, Record, Ty,
      },
      Record as _,
    },
    misc::Vector,
  };
  use core::marker::PhantomData;

  #[test]
  fn returns_correct_values() {
    let bytes = &[0, 0, 0, 1, 1, 0, 0, 0, 2, 2, 3, 0, 0, 0, 1, 4];
    let values = &[(column0(), Ty::Any), (column1(), Ty::Any), (column2(), Ty::Any)];
    let mut values_bytes_offsets = Vector::new();
    let stmt = Statement::new(3, 0, values);
    let record = Record::<crate::Error>::parse(
      bytes,
      0..bytes.len(),
      stmt.clone(),
      &mut values_bytes_offsets,
      3,
    )
    .unwrap();
    assert_eq!(
      record,
      Record {
        bytes: &[1, 0, 0, 0, 2, 2, 3, 0, 0, 0, 1, 4],
        initial_value_offset: 0,
        stmt,
        values_bytes_offsets: &[(false, 0usize..1usize), (false, 5..7), (false, 11..12)],
        phantom: PhantomData
      }
    );
    assert_eq!(record.len(), 3);
    assert_eq!(record.value(0), Some(DecodeValue::new(&[1][..], column0().ty)));
    assert_eq!(record.value(1), Some(DecodeValue::new(&[2, 3][..], column1().ty)));
    assert_eq!(record.value(2), Some(DecodeValue::new(&[4][..], column2().ty)));
    assert_eq!(record.value(3), None);
  }
}
