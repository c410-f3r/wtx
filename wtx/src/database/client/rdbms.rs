pub(crate) mod common_executor_buffer;
pub(crate) mod common_record;
pub(crate) mod common_records;
pub(crate) mod statement;
pub(crate) mod statement_builder;
pub(crate) mod statements;
pub(crate) mod statements_misc;

use crate::{
  collection::Vector,
  database::{Database, ValueIdent, client::rdbms::common_record::CommonRecord},
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
pub(crate) fn value<'any, 'exec, A, C, CI, D, R, T>(
  ci: CI,
  record: &R,
) -> Option<<D as DEController>::DecodeWrapper<'exec, 'any>>
where
  C: Lease<D::Ty> + 'exec,
  CI: ValueIdent<R>,
  D: Database,
  D::Ty: Clone,
  R: Lease<CommonRecord<'exec, A, C, D, T>>,
  T: 'exec,
  <D as DEController>::DecodeWrapper<'exec, 'any>: From<(&'exec [u8], D::Ty)>,
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
    Some(From::from((bytes, column.lease().clone())))
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
