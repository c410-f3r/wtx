use crate::{
  http::{Headers, Method, Request, Response, StatusCode, Version},
  misc::{Lease, LeaseMut, UriRef, Vector, VectorError},
};
use core::str::from_utf8_unchecked;

/// Buffer used for requests or responses.
#[derive(Debug)]
pub struct ReqResBuffer {
  data: Vector<u8>,
  headers: Headers,
  uri_authority_start_idx: u8,
  uri_href_start_idx: u16,
  uri_start_idx: u16,
}

impl ReqResBuffer {
  /// Gathers all necessary elements to send and receive low-level requests/responses.
  #[inline]
  pub const fn new(data: Vector<u8>, headers: Headers) -> Self {
    Self { data, headers, uri_authority_start_idx: 0, uri_href_start_idx: 0, uri_start_idx: 0 }
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

  /// Data associated with the request/response.
  #[inline]
  pub fn body(&self) -> &[u8] {
    Self::do_body(&self.data, self.uri_start_idx)
  }

  /// Clears all buffers, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    let Self {
      uri_authority_start_idx: authority_start_idx,
      data: body,
      headers,
      uri_href_start_idx: href_start_idx,
      uri_start_idx,
    } = self;
    *authority_start_idx = 0;
    body.clear();
    headers.clear();
    *href_start_idx = 0;
    *uri_start_idx = 0;
  }

  /// Clears the internal buffer that composes a request/response body.
  #[inline]
  pub fn clear_body(&mut self) {
    self.data.truncate(self.uri_start_idx.into());
  }

  /// Clears the internal buffer that composes a request/response body as well as the headers.
  #[inline]
  pub fn clear_body_and_headers(&mut self) {
    self.clear_body();
    self.headers.clear();
  }

  /// Adds more bytes to the body of the request/response.
  #[inline]
  pub fn extend_body(&mut self, value: &[u8]) -> Result<(), VectorError> {
    self.data.extend_from_slice(value)
  }

  /// See [Headers].
  #[inline]
  pub(crate) fn headers(&self) -> &Headers {
    &self.headers
  }

  /// Mutable version of [`Self::headers`].
  #[inline]
  pub fn headers_mut(&mut self) -> &mut Headers {
    &mut self.headers
  }

  /// Consumes itself to provide the underlying elements used to perform operations.
  #[inline]
  pub fn into_parts(self) -> (Vector<u8>, Headers) {
    (self.data, self.headers)
  }

  /// Mutable version of [`Self::into_parts`].
  #[inline]
  pub fn parts_mut(&mut self) -> (&mut [u8], &mut Headers) {
    (Self::do_body_mut(&mut self.data, self.uri_start_idx), &mut self.headers)
  }

  /// Clears the internal data (including the request/response body) and sets a new URI based on
  /// the three fundamental string parts that compose an URI.
  #[inline]
  pub fn set_uri_from_parts(
    &mut self,
    scheme: &str,
    authority: &str,
    path: &str,
  ) -> Result<(), VectorError> {
    self.set_uri(|this| {
      this.data.extend_from_slices(&[scheme.as_bytes(), authority.as_bytes(), path.as_bytes()])
    })
  }

  /// Clears the internal data (including the request/response body) and sets a new URI based on a
  /// string.
  #[inline]
  pub fn set_uri_from_str(&mut self, str: &str) -> Result<(), VectorError> {
    self.set_uri(|this| this.data.extend_from_slice(str.as_bytes()))
  }

  /// URI of a ***request***. Returns an empty [`UriRef`] otherwise.
  #[inline]
  pub fn uri(&self) -> UriRef<'_> {
    UriRef::from_parts(self.uri_str(), self.uri_authority_start_idx, self.uri_href_start_idx)
  }

  /// URI string of a ***request***. Returns an empty string otherwise.
  #[inline]
  pub fn uri_str(&self) -> &str {
    Self::do_uri_str(&self.data, self.uri_start_idx)
  }

  #[inline]
  fn do_body(data: &Vector<u8>, uri_start_idx: u16) -> &[u8] {
    // SAFETY: `uri_start_idx` is not publicly exposed and is always within bounds
    unsafe { data.get(uri_start_idx.into()..).unwrap_unchecked() }
  }

  #[inline]
  fn do_body_mut(data: &mut Vector<u8>, uri_start_idx: u16) -> &mut [u8] {
    // SAFETY: `uri_start_idx` is not publicly exposed and is always within bounds
    unsafe { data.get_mut(uri_start_idx.into()..).unwrap_unchecked() }
  }

  #[inline]
  fn do_uri_str(data: &Vector<u8>, uri_start_idx: u16) -> &str {
    // SAFETY: `uri_start_idx` is not publicly exposed and is always within bounds
    let slice = unsafe { data.get(..uri_start_idx.into()).unwrap_unchecked() };
    // SAFETY: `data` is not publicly exposed and the URI part always contain valid UTF-8
    unsafe { from_utf8_unchecked(slice) }
  }

  #[inline]
  fn set_uri(
    &mut self,
    cb: impl FnOnce(&mut Self) -> Result<(), VectorError>,
  ) -> Result<(), VectorError> {
    self.data.clear();
    cb(self)?;
    let uri_start_idx = if let Ok(elem) = self.data.len().try_into() {
      elem
    } else {
      let len = u16::MAX;
      self.data.truncate(len.into());
      len
    };
    let uri = UriRef::new(Self::do_uri_str(&self.data, uri_start_idx));
    self.uri_authority_start_idx = uri.authority_start_idx();
    self.uri_href_start_idx = uri.href_start_idx();
    self.uri_start_idx = uri_start_idx;
    Ok(())
  }
}

impl Default for ReqResBuffer {
  #[inline]
  fn default() -> Self {
    Self {
      uri_authority_start_idx: 0,
      data: Vector::new(),
      headers: Headers::new(0),
      uri_href_start_idx: 0,
      uri_start_idx: 0,
    }
  }
}

impl Lease<[u8]> for ReqResBuffer {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.body()
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
