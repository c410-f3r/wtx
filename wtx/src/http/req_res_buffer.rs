use crate::{
  http::{Headers, Method, ReqResData, ReqResDataMut, Request, Response, StatusCode, Version},
  misc::{Lease, LeaseMut, UriString, Vector},
};
use alloc::string::String;

/// Buffer used for requests or responses.
#[derive(Debug)]
pub struct ReqResBuffer {
  /// See [`Vector`].
  pub body: Vector<u8>,
  /// See [`Headers`].
  pub headers: Headers,
  /// See [`UriString`].
  pub uri: UriString,
}

impl ReqResBuffer {
  /// Empty instance
  #[inline]
  pub const fn empty() -> Self {
    Self::new(Vector::new(), Headers::new(), UriString::_empty(String::new()))
  }

  /// Constructor shortcut
  #[inline]
  pub const fn new(data: Vector<u8>, headers: Headers, uri: UriString) -> Self {
    Self { body: data, headers, uri }
  }

  /// Shortcut to create a HTTP/2 [Request].
  #[inline]
  pub fn as_http2_request(&self, method: Method) -> Request<&Self> {
    Request { method, rrd: self, version: Version::Http2 }
  }

  /// Mutable version of [`Self::as_http2_request`].
  #[inline]
  pub fn as_http2_request_mut(&mut self, method: Method) -> Request<&mut Self> {
    Request { method, rrd: self, version: Version::Http2 }
  }

  /// Shortcut to create a HTTP/2 [Response].
  #[inline]
  pub fn as_http2_response(&self, status_code: StatusCode) -> Response<&Self> {
    Response { rrd: self, status_code, version: Version::Http2 }
  }

  /// Mutable version of [`Self::as_http2_response`].
  #[inline]
  pub fn as_http2_response_mut(&mut self, status_code: StatusCode) -> Response<&mut Self> {
    Response { rrd: self, status_code, version: Version::Http2 }
  }

  /// Clears all buffers, removing all values.
  ///
  /// The internal vector as well as the internal headers are returned in a valid state.
  #[inline]
  pub fn clear(&mut self) {
    let Self { body: data, headers, uri } = self;
    data.clear();
    headers.clear();
    uri.clear();
  }

  /// Owned version of [`Self::as_http2_request`].
  #[inline]
  pub fn into_http2_request(self, method: Method) -> Request<Self> {
    Request { method, rrd: self, version: Version::Http2 }
  }

  /// Owned version of [`Self::as_http2_response`].
  #[inline]
  pub fn into_http2_response(self, status_code: StatusCode) -> Response<Self> {
    Response { rrd: self, status_code, version: Version::Http2 }
  }

  /// Mutable parts
  #[inline]
  pub fn parts_mut(&mut self) -> (&mut Vector<u8>, &mut Headers, &mut UriString) {
    (&mut self.body, &mut self.headers, &mut self.uri)
  }
}

impl ReqResData for ReqResBuffer {
  type Body = Vector<u8>;

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.body
  }

  #[inline]
  fn headers(&self) -> &Headers {
    &self.headers
  }

  #[inline]
  fn uri(&self) -> &UriString {
    &self.uri
  }
}

impl ReqResDataMut for ReqResBuffer {
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    &mut self.body
  }

  #[inline]
  fn clear(&mut self) {
    self.body.clear();
    self.headers.clear();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    &mut self.headers
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString) {
    (&mut self.body, &mut self.headers, &self.uri)
  }
}

impl Lease<[u8]> for ReqResBuffer {
  #[inline]
  fn lease(&self) -> &[u8] {
    &self.body
  }
}

impl Lease<ReqResBuffer> for ReqResBuffer {
  #[inline]
  fn lease(&self) -> &ReqResBuffer {
    self
  }
}

impl LeaseMut<ReqResBuffer> for ReqResBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut ReqResBuffer {
    self
  }
}

impl Default for ReqResBuffer {
  #[inline]
  fn default() -> Self {
    Self::empty()
  }
}

impl From<Vector<u8>> for ReqResBuffer {
  #[inline]
  fn from(from: Vector<u8>) -> Self {
    Self { body: from, headers: Headers::new(), uri: UriString::_empty(String::new()) }
  }
}

#[cfg(feature = "std")]
impl core::fmt::Write for ReqResBuffer {
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.body.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| core::fmt::Error)
  }
}

#[cfg(feature = "std")]
impl std::io::Write for ReqResBuffer {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.body.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.body.flush()
  }
}
