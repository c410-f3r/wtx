use crate::{
  http::Headers,
  http2::uri_buffer::MAX_URI_LEN,
  misc::{ArrayString, ByteVector},
};
use alloc::boxed::Box;

/// Buffer used for requests or responses.
#[derive(Debug)]
pub struct ReqResBuffer {
  /// See [ByteVector].
  pub data: ByteVector,
  /// See [Headers].
  pub headers: Headers,
  /// Scheme, authority and path.
  pub uri: Box<ArrayString<{ MAX_URI_LEN }>>,
}

impl ReqResBuffer {
  pub(crate) fn clear(&mut self) {
    self.data.clear();
    self.headers.clear();
  }
}

impl Default for ReqResBuffer {
  #[inline]
  fn default() -> Self {
    Self { data: ByteVector::with_capacity(1024), headers: Headers::default(), uri: Box::default() }
  }
}
