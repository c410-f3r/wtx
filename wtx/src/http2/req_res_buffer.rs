use crate::{
  http::{Headers, Method, RequestRef, Version},
  http2::{uri_buffer::MAX_URI_LEN, CACHED_HEADERS_LEN_LOWER_BOUND},
  misc::{ArrayString, ByteVector, UriRef, Usize},
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
    let n = *Usize::from(CACHED_HEADERS_LEN_LOWER_BOUND);
    Self { data: ByteVector::new(), headers: Headers::new(n), uri: Box::new(ArrayString::new()) }
  }
}

impl ReqResBuffer {
  #[inline]
  pub(crate) fn clear(&mut self) {
    self.data.clear();
    self.headers.clear();
  }

  /// Shortcut to create a [RequestRef] with inner data.
  pub fn as_http2_request_ref(&self, method: Method) -> RequestRef<'_, '_, '_, [u8]> {
    RequestRef {
      data: &self.data,
      headers: &self.headers,
      method,
      uri: UriRef::new(self.uri.as_str()),
      version: Version::Http2,
    }
  }
}
