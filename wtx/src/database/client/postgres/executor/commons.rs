use crate::{
  database::client::postgres::Ty,
  misc::{ConnectionState, Vector},
};

pub(crate) struct FetchWithStmtCommons<'others, S> {
  pub(crate) cs: &'others mut ConnectionState,
  pub(crate) rb: &'others mut Vector<usize>,
  pub(crate) stream: &'others mut S,
  /// Pre-specified types
  pub(crate) tys: &'others [Ty],
}
