use crate::{
  codec::CodecController,
  collections::Vector,
  database::{
    Record, ValueIdent,
    client::{
      postgres::{Postgres, PostgresCommonRecord, PostgresError, PostgresStatement},
      rdbms::value,
    },
  },
  misc::Lease,
};
use core::ops::Range;

/// Record
#[derive(Debug)]
pub struct PostgresRecord<'exec, E> {
  pub(crate) common: PostgresCommonRecord<'exec, E>,
}

impl<'exec, E> PostgresRecord<'exec, E> {
  pub(crate) const fn new(
    record: &'exec [u8],
    stmt: PostgresStatement<'exec>,
    values_params: &'exec [(bool, Range<usize>)],
  ) -> Self {
    Self { common: PostgresCommonRecord::new(record, stmt, values_params) }
  }

  pub(crate) fn parse(
    record: &'exec [u8],
    stmt: PostgresStatement<'exec>,
    values_len: u16,
    values_params: &'exec mut Vector<(bool, Range<usize>)>,
  ) -> crate::Result<Self> {
    fn fun(
      [b0, b1, b2, b3]: [u8; 4],
      curr_value_offset: &mut usize,
      values_params: &mut Vector<(bool, Range<usize>)>,
    ) -> crate::Result<()> {
      let begin = *curr_value_offset;
      let n = i32::from_be_bytes([b0, b1, b2, b3]);
      let (is_null, end) = match n {
        -1 => (true, begin),
        _ => (false, begin.wrapping_add(usize::try_from(n)?)),
      };
      values_params.push((is_null, begin..end))?;
      *curr_value_offset = end;
      crate::Result::Ok(())
    }

    let values_bytes_offsets_start = values_params.len();
    let mut curr_value_offset = 4;

    match (record, values_len) {
      ([b0, b1, b2, b3, ..], 1..=u16::MAX) => {
        fun([*b0, *b1, *b2, *b3], &mut curr_value_offset, values_params)?;
      }
      _ => {
        return Ok(Self::new(record, stmt, values_params));
      }
    }

    for _ in 1..values_len {
      let Some(&[b0, b1, b2, b3, ..]) = record.get(curr_value_offset..) else {
        return Err(PostgresError::InvalidPostgresRecord.into());
      };
      curr_value_offset = curr_value_offset.wrapping_add(4);
      fun([b0, b1, b2, b3], &mut curr_value_offset, values_params)?;
    }

    Ok(Self::new(record, stmt, values_params.get(values_bytes_offsets_start..).unwrap_or_default()))
  }
}

impl<'exec, E> Lease<PostgresCommonRecord<'exec, E>> for PostgresRecord<'exec, E> {
  #[inline]
  fn lease(&self) -> &PostgresCommonRecord<'exec, E> {
    &self.common
  }
}

impl<'exec, E> Record<'exec> for PostgresRecord<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Postgres<E>;

  #[inline]
  fn len(&self) -> usize {
    self.common.values_params.len()
  }

  #[inline]
  fn value<CI>(
    &self,
    ci: CI,
  ) -> Option<<Self::Database as CodecController>::DecodeWrapper<'exec, '_, '_>>
  where
    CI: ValueIdent<Self>,
  {
    value(ci, self)
  }
}

impl<'exec, E> ValueIdent<PostgresRecord<'exec, E>> for &str {
  #[inline]
  fn idx(&self, input: &PostgresRecord<'exec, E>) -> Option<usize> {
    self.idx(&input.common)
  }
}

impl<'exec, E> From<PostgresCommonRecord<'exec, E>> for PostgresRecord<'exec, E> {
  #[inline]
  fn from(from: PostgresCommonRecord<'exec, E>) -> Self {
    Self { common: from }
  }
}
