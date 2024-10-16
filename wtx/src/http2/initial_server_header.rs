use crate::{
  http::{Method, ReqResBuffer},
  http2::U31,
};
use core::task::Waker;

#[derive(Debug)]
pub(crate) struct InitialServerHeader {
  pub(crate) method: Method,
  pub(crate) rrb: ReqResBuffer,
  pub(crate) stream_id: U31,
  pub(crate) waker: Waker,
}
