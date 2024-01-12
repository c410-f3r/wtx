use crate::database::client::postgres::Ty;
use alloc::vec::Vec;
use hashbrown::HashMap;

pub(crate) struct FetchWithStmtCommons<'others, S> {
  pub(crate) ftb: &'others mut Vec<(usize, u32)>,
  pub(crate) is_closed: &'others mut bool,
  pub(crate) rb: &'others mut Vec<usize>,
  pub(crate) stream: &'others mut S,
  pub(crate) tb: &'others mut HashMap<u32, Ty>,
  pub(crate) tys: &'others [Ty],
}
