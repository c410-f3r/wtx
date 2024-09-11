use crate::{
  http::{Headers, Method, ReqResData, ReqResDataMut, Request, Response, StatusCode, Version},
  misc::{Lease, LeaseMut, UriRef, UriString, Vector},
};
use alloc::string::String;

/// Buffer used for requests or responses.
#[derive(Debug)]
pub struct ReqResBuffer {
  /// See [`Vector`].
  pub data: Vector<u8>,
  /// See [`Headers`].
  pub headers: Headers,
  /// See [`UriString`].
  pub uri: UriString,
}

impl ReqResBuffer {
  pub(crate) const fn empty() -> Self {
    ReqResBuffer::new(Vector::new(), Headers::new(), UriString::_empty(String::new()))
  }

  /// Constructor shortcut
  #[inline]
  pub const fn new(data: Vector<u8>, headers: Headers, uri: UriString) -> Self {
    Self { data, headers, uri }
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
    let Self { data, headers, uri } = self;
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
}

impl ReqResData for ReqResBuffer {
  type Body = Vector<u8>;

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.data
  }

  #[inline]
  fn headers(&self) -> &Headers {
    &self.headers
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    self.uri.to_ref()
  }
}

impl ReqResDataMut for ReqResBuffer {
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    &mut self.data
  }

  #[inline]
  fn clear(&mut self) {
    self.data.clear();
    self.headers.clear();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    &mut self.headers
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (&mut self.data, &mut self.headers, self.uri.to_ref())
  }
}

impl Default for ReqResBuffer {
  #[inline]
  fn default() -> Self {
    Self::empty()
  }
}

impl Lease<[u8]> for ReqResBuffer {
  #[inline]
  fn lease(&self) -> &[u8] {
    &self.data
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

#[cfg(feature = "std")]
impl core::fmt::Write for ReqResBuffer {
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.data.extend_from_slice(s.as_bytes()).map_err(|_err| core::fmt::Error)
  }
}

#[cfg(feature = "std")]
impl std::io::Write for ReqResBuffer {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.data.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.data.flush()
  }
}
