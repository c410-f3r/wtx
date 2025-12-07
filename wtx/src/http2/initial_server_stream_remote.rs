use crate::{
  http::{Method, Protocol},
  http2::u31::U31,
};

#[derive(Debug)]
pub(crate) struct InitialServerStreamRemote {
  pub(crate) method: Method,
  pub(crate) protocol: Option<Protocol>,
  pub(crate) stream_id: U31,
}
