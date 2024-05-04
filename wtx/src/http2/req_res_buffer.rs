use crate::{
  http::Headers,
  http2::uri_buffer::MAX_URI_LEN,
  misc::{ArrayString, ByteVector},
};
use alloc::boxed::Box;

/// Buffer used for requests or responses.
//
// Maximum sizes are dictated by `AcceptParams` or `ConnectParams`.
#[derive(Debug)]
pub struct ReqResBuffer {
  /// See [ByteVector].
  pub data: ByteVector,
  /// See [Headers].
  pub headers: Headers,
  /// Scheme, authority and path.
  pub uri: Box<ArrayString<{ MAX_URI_LEN }>>,
}

impl Default for ReqResBuffer {
  fn default() -> Self {
    Self { data: ByteVector::new(), headers: Headers::new(0), uri: Box::new(ArrayString::new()) }
  }
}

impl ReqResBuffer {
  #[inline]
  pub(crate) fn clear(&mut self) {
    self.data.clear();
    self.headers.clear();
  }
}
