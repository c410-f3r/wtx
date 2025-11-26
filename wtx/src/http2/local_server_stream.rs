use crate::{
  http::{Method, Protocol, ReqResBuffer},
  http2::u31::U31,
};
use core::task::Waker;

#[derive(Debug)]
pub(crate) struct LocalServerStream {
  pub(crate) method: Method,
  pub(crate) protocol: Option<Protocol>,
  pub(crate) rrb: ReqResBuffer,
  pub(crate) stream_id: U31,
  pub(crate) waker: Waker,
}
