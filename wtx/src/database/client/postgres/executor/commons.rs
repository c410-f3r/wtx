use crate::{database::client::postgres::Ty, misc::ConnectionState};

pub(crate) struct FetchWithStmtCommons<'others, S> {
  pub(crate) cs: &'others mut ConnectionState,
  pub(crate) stream: &'others mut S,
  /// Pre-specified types
  pub(crate) tys: &'others [Ty],
}
