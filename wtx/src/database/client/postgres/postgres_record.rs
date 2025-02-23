use crate::{
  database::{
    Record, ValueIdent,
    client::postgres::{DecodeWrapper, Postgres, PostgresError, PostgresStatement},
  },
  misc::{_unlikely_dflt, _unlikely_elem, DEController, Vector},
};
use core::{marker::PhantomData, ops::Range};

/// Record
#[derive(Debug)]
pub struct PostgresRecord<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) initial_value_offset: usize,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stmt: PostgresStatement<'exec>,
  pub(crate) values_bytes_offsets: &'exec [(bool, Range<usize>)],
}

impl<'exec, E> PostgresRecord<'exec, E> {
  #[inline]
  pub(crate) fn new(
    bytes: &'exec [u8],
    initial_value_offset: usize,
    stmt: PostgresStatement<'exec>,
    values_bytes_offsets: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { bytes, initial_value_offset, phantom: PhantomData, stmt, values_bytes_offsets }
  }

  pub(crate) fn parse(
    mut bytes: &'exec [u8],
    bytes_range: Range<usize>,
    stmt: PostgresStatement<'exec>,
    values_bytes_offsets: &'exec mut Vector<(bool, Range<usize>)>,
    values_len: u16,
  ) -> crate::Result<Self> {
    #[inline]
    fn fun(
      [a, b, c, d]: [u8; 4],
      curr_value_offset: &mut usize,
      values_bytes_offsets: &mut Vector<(bool, Range<usize>)>,
    ) -> crate::Result<()> {
      let begin = *curr_value_offset;
      let n = i32::from_be_bytes([a, b, c, d]);
      let (is_null, end) = match n {
        -1 => (true, begin),
        _ => (false, begin.wrapping_add(usize::try_from(n)?)),
      };
      values_bytes_offsets.push((is_null, begin..end))?;
      *curr_value_offset = end;
      crate::Result::Ok(())
    }

    let values_bytes_offsets_start = values_bytes_offsets.len();
    let initial_value_offset = bytes_range.start;
    let mut curr_value_offset = bytes_range.start;

    let local_bytes = bytes.get(bytes_range);
    match (local_bytes, values_len) {
      (Some([a, b, c, d, rest @ ..]), 1..=u16::MAX) => {
        bytes = rest;
        fun([*a, *b, *c, *d], &mut curr_value_offset, values_bytes_offsets)?;
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
      fun([a, b, c, d], &mut curr_value_offset, values_bytes_offsets)?;
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

impl<'exec, E> Record<'exec> for PostgresRecord<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn len(&self) -> usize {
    self.values_bytes_offsets.len()
  }

  #[inline]
  fn value<CI>(&self, ci: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'exec, '_>>
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
      let column = match self.stmt._column(idx) {
        None => return _unlikely_elem(None),
        Some(elem) => elem,
      };
      let end = range.end.wrapping_sub(self.initial_value_offset);
      let bytes = match self.bytes.get(begin..end) {
        None => return _unlikely_elem(None),
        Some(elem) => elem,
      };
      Some(DecodeWrapper::new(bytes, column.ty))
    }
  }
}

impl<'exec, E> ValueIdent<PostgresRecord<'exec, E>> for str {
  #[inline]
  fn idx(&self, input: &PostgresRecord<'exec, E>) -> Option<usize> {
    input.stmt._columns().position(|column| column.name.as_str() == self)
  }
}

impl<E> PartialEq for PostgresRecord<'_, E> {
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
    database::{FromRecord, Record, client::postgres::Postgres},
    misc::{ArrayString, from_utf8_basic, into_rslt},
  };

  impl<E, const N: usize> FromRecord<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn from_record(
      record: &crate::database::client::postgres::postgres_record::PostgresRecord<'_, E>,
    ) -> Result<Self, E> {
      Ok(from_utf8_basic(into_rslt(record.value(0))?.bytes()).map_err(From::from)?.try_into()?)
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    database::{
      Record as _,
      client::postgres::{
        DecodeWrapper, PostgresRecord, PostgresStatement, Ty,
        tests::{column0, column1, column2},
      },
    },
    misc::Vector,
  };

  #[test]
  fn returns_correct_values() {
    let bytes = &[0, 0, 0, 1, 1, 0, 0, 0, 2, 2, 3, 0, 0, 0, 1, 4];
    let values = &[(column0(), Ty::Any), (column1(), Ty::Any), (column2(), Ty::Any)];
    let mut values_bytes_offsets = Vector::new();
    let stmt = PostgresStatement::new((), 3, 0, values);
    let record = PostgresRecord::<crate::Error>::parse(
      bytes,
      0..bytes.len(),
      stmt.clone(),
      &mut values_bytes_offsets,
      3,
    )
    .unwrap();
    assert_eq!(
      record,
      PostgresRecord::new(
        &[1, 0, 0, 0, 2, 2, 3, 0, 0, 0, 1, 4],
        0,
        stmt,
        &[(false, 0usize..1usize), (false, 5..7), (false, 11..12)],
      )
    );
    assert_eq!(record.len(), 3);
    assert_eq!(record.value(0), Some(DecodeWrapper::new(&[1][..], column0().ty)));
    assert_eq!(record.value(1), Some(DecodeWrapper::new(&[2, 3][..], column1().ty)));
    assert_eq!(record.value(2), Some(DecodeWrapper::new(&[4][..], column2().ty)));
    assert_eq!(record.value(3), None);
  }
}
