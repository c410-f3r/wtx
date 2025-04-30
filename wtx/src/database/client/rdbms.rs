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
  misc::{
    DEController, Lease, UriRef, hints::unlikely_elem, net::PartitionedFilledBuffer,
    str_split_once1, str_split1,
  },
};
use core::ops::Range;

/// Should be called before executing commands.
#[inline]
pub(crate) fn clear_cmd_buffers(
  net_buffer: &mut PartitionedFilledBuffer,
  records_params: &mut Vector<(Range<usize>, Range<usize>)>,
  values_params: &mut Vector<(bool, Range<usize>)>,
) {
  net_buffer._clear_if_following_is_empty();
  records_params.clear();
  values_params.clear();
}

#[inline]
pub(crate) fn query_walker<'uri>(
  uri: &'uri UriRef<'_>,
  mut cb: impl FnMut(&'uri str, &'uri str) -> crate::Result<()>,
) -> crate::Result<()> {
  let mut pair_iter = str_split1(uri.query_and_fragment(), b'&');
  if let Some(mut key_value) = pair_iter.next() {
    key_value = key_value.get(1..).unwrap_or_default();
    if let Some((key, value)) = str_split_once1(key_value, b'=') {
      cb(key, value)?;
    }
  }
  for key_value in pair_iter {
    if let Some((key, value)) = str_split_once1(key_value, b'=') {
      cb(key, value)?;
    }
  }
  Ok(())
}

// FIXME(STABLE): CommonRecord should implement Record but in such a scenario GAT implies
// static bounds.
#[inline]
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
    None => return unlikely_elem(None),
    Some(elem) => elem,
  };
  if *is_null {
    None
  } else {
    let column = match record.lease().stmt._column(idx) {
      None => return unlikely_elem(None),
      Some(elem) => elem,
    };
    let bytes = match record.lease().record.get(range.clone()) {
      None => return unlikely_elem(None),
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
