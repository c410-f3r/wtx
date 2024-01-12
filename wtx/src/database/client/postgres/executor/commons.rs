use crate::database::client::postgres::Ty;
use alloc::vec::Vec;

pub(crate) struct FetchWithStmtCommons<'others, S> {
  pub(crate) is_closed: &'others mut bool,
  pub(crate) rb: &'others mut Vec<usize>,
  pub(crate) stream: &'others mut S,
  pub(crate) tys: &'others [Ty],
}
