pub(crate) mod column_info;
pub(crate) mod common_client_buffer;
pub(crate) mod common_record;
pub(crate) mod common_records;
pub(crate) mod statement;
pub(crate) mod statement_builder;
pub(crate) mod statements;
pub(crate) mod statements_misc;

use crate::{
  codec::CodecController,
  collections::Vector,
  database::{
    Database, ValueIdent,
    client::rdbms::{column_info::ColumnInfo, common_record::CommonRecord},
  },
  misc::Lease,
};
use core::ops::Range;

/// Should be called before executing commands.
pub(crate) fn clear_query_buffers(
  records_params: &mut Vector<(Range<usize>, Range<usize>)>,
  values_params: &mut Vector<(bool, Range<usize>)>,
) {
  records_params.clear();
  values_params.clear();
}

// FIXME(STABLE): CommonRecord should implement Record but in such a scenario GAT implies
// static bounds.
pub(crate) fn value<'bytes, 'rem, A, C, CI, D, R, T>(
  ci: CI,
  record: &'rem R,
) -> Option<<D as CodecController>::DecodeWrapper<'bytes, 'bytes, 'rem>>
where
  A: 'rem,
  C: ColumnInfo<Ty = D::Ty> + 'bytes + 'rem,
  CI: ValueIdent<R>,
  D: Database + 'rem,
  D::Ty: Clone,
  R: Lease<CommonRecord<'bytes, A, C, D, T>>,
  T: 'bytes,
  <D as CodecController>::DecodeWrapper<'bytes, 'bytes, 'rem>: From<(&'bytes [u8], &'rem C)>,
  'bytes: 'rem,
{
  let idx = ci.idx(record)?;
  let (is_null, range) = record.lease().values_params.get(idx)?;
  if *is_null {
    None
  } else {
    let column = record.lease().stmt.column(idx)?;
    let bytes = record.lease().record.get(range.clone())?;
    Some(From::from((bytes, column)))
  }
}

#[cfg(test)]
pub(crate) mod tests {
  pub(crate) fn _column0() -> &'static str {
    "a"
  }

  pub(crate) fn _column1() -> &'static str {
    "b"
  }

  pub(crate) fn _column2() -> &'static str {
    "c"
  }

  pub(crate) fn _column3() -> &'static str {
    "d"
  }
}
