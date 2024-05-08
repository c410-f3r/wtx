use crate::{
  http::{Headers, Method, Request, Version},
  http2::uri_buffer::MAX_URI_LEN,
  misc::{ArrayString, ByteVector, UriRef},
};
use alloc::boxed::Box;

/// Buffer used for requests or responses.
//
// Maximum sizes are dictated by `AcceptParams` or `ConnectParams`.
#[derive(Debug)]
pub struct ReqResBuffer {
  /// See [ByteVector].
  pub body: ByteVector,
  /// See [Headers].
  pub headers: Headers,
  /// Scheme, authority and path.
  pub uri: Box<ArrayString<{ MAX_URI_LEN }>>,
}

// For servers, the default headers length must be used until a settings frame is received.
impl Default for ReqResBuffer {
  fn default() -> Self {
    Self { body: ByteVector::new(), headers: Headers::new(0), uri: Box::new(ArrayString::new()) }
  }
}

impl ReqResBuffer {
  /// Shortcut that avoids having to call `with_capacity` on each field.
  ///
  /// Should be used if you are willing to manually push data.
  pub fn with_capacity(
    body: usize,
    headers_bytes: usize,
    headers_headers: usize,
    headers_max_bytes: usize,
  ) -> Self {
    Self {
      body: ByteVector::with_capacity(body),
      headers: Headers::with_capacity(headers_bytes, headers_headers, headers_max_bytes),
      uri: Box::new(ArrayString::new()),
    }
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    self.body.clear();
    self.headers.clear();
  }

  /// Shortcut to create a [RequestRef] with inner body.
  pub fn as_http2_request(&self, method: Method) -> Request<(&ByteVector, &Headers), &str> {
    Request {
      data: (&self.body, &self.headers),
      method,
      uri: UriRef::new(self.uri.as_str()),
      version: Version::Http2,
    }
  }
}
