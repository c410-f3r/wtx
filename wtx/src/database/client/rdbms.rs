pub(crate) mod column_info;
pub(crate) mod common_executor_buffer;
pub(crate) mod common_record;
pub(crate) mod common_records;
pub(crate) mod statement;
pub(crate) mod statement_builder;
pub(crate) mod statements;
pub(crate) mod statements_misc;

use crate::{
  collection::Vector,
  database::{
    Database, ValueIdent,
    client::rdbms::{column_info::ColumnInfo, common_record::CommonRecord},
  },
  de::DEController,
  misc::{Lease, hints::_unlikely_elem, net::PartitionedFilledBuffer},
};
use core::ops::Range;

/// Should be called before executing commands.
pub(crate) fn clear_cmd_buffers(
  net_buffer: &mut PartitionedFilledBuffer,
  records_params: &mut Vector<(Range<usize>, Range<usize>)>,
  values_params: &mut Vector<(bool, Range<usize>)>,
) {
  net_buffer.clear_if_following_is_empty();
  records_params.clear();
  values_params.clear();
}

// FIXME(STABLE): CommonRecord should implement Record but in such a scenario GAT implies
// static bounds.
pub(crate) fn value<'inner, 'outer, 'rem, A, C, CI, D, R, T>(
  ci: CI,
  record: &'rem R,
) -> Option<<D as DEController>::DecodeWrapper<'inner, 'outer, 'rem>>
where
  A: 'rem,
  C: ColumnInfo<Ty = D::Ty> + 'inner + 'rem,
  CI: ValueIdent<R>,
  D: Database + 'rem,
  D::Ty: Clone,
  R: Lease<CommonRecord<'inner, A, C, D, T>>,
  T: 'inner,
  <D as DEController>::DecodeWrapper<'inner, 'outer, 'rem>: From<(&'inner [u8], &'rem C)>,
  'inner: 'rem,
{
  let idx = ci.idx(record)?;
  let (is_null, range) = match record.lease().values_params.get(idx) {
    None => return _unlikely_elem(None),
    Some(elem) => elem,
  };
  if *is_null {
    None
  } else {
    let column = match record.lease().stmt.column(idx) {
      None => return _unlikely_elem(None),
      Some(elem) => elem,
    };
    let bytes = match record.lease().record.get(range.clone()) {
      None => return _unlikely_elem(None),
      Some(elem) => elem,
    };
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
