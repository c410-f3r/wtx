use crate::{
  collection::{Clear, Vector},
  http::{Headers, Method, MsgData, MsgDataMut, Request, Response, StatusCode},
  misc::{Lease, LeaseMut, Uri, UriRef},
};
use alloc::string::String;
use core::fmt::{Debug, Formatter};

/// A request or a response where the URI is an immutable string slice.
pub type MsgBufferStr<'uri> = MsgBuffer<&'uri str>;
/// A request or a response where the URI is a dynamic string buffer.
pub type MsgBufferString = MsgBuffer<String>;

/// An HTTP message buffer can refer a request or a response.
///
/// Buffer used for requests or responses.
pub struct MsgBuffer<S> {
  /// See [`Vector`].
  pub body: Vector<u8>,
  /// See [`Headers`].
  pub headers: Headers,
  /// Generic URI
  pub uri: Uri<S>,
}

impl<S> MsgBuffer<S> {
  /// Empty instance
  #[inline]
  pub const fn from_uri(uri: Uri<S>) -> Self {
    Self::new(Vector::new(), Headers::new(), uri)
  }

  /// Constructor shortcut
  #[inline]
  pub const fn new(data: Vector<u8>, headers: Headers, uri: Uri<S>) -> Self {
    Self { body: data, headers, uri }
  }

  /// Shortcut to create a HTTP/2 [Request].
  #[inline]
  pub const fn as_http2_request(&self, method: Method) -> Request<&Self> {
    Request { method, msg_data: self }
  }

  /// Mutable version of [`Self::as_http2_request`].
  #[inline]
  pub const fn as_http2_request_mut(&mut self, method: Method) -> Request<&mut Self> {
    Request { method, msg_data: self }
  }

  /// Shortcut to create a HTTP/2 [Response].
  #[inline]
  pub const fn as_http2_response(&self, status_code: StatusCode) -> Response<&Self> {
    Response { msg_data: self, status_code }
  }

  /// Mutable version of [`Self::as_http2_response`].
  #[inline]
  pub const fn as_http2_response_mut(&mut self, status_code: StatusCode) -> Response<&mut Self> {
    Response { msg_data: self, status_code }
  }

  /// Owned version of [`Self::as_http2_request`].
  #[inline]
  pub const fn into_http2_request(self, method: Method) -> Request<Self> {
    Request { method, msg_data: self }
  }

  /// Owned version of [`Self::as_http2_response`].
  #[inline]
  pub const fn into_http2_response(self, status_code: StatusCode) -> Response<Self> {
    Response { msg_data: self, status_code }
  }

  /// Mutable parts
  #[inline]
  pub const fn parts_mut(&mut self) -> (&mut Vector<u8>, &mut Headers, &mut Uri<S>) {
    (&mut self.body, &mut self.headers, &mut self.uri)
  }
}

impl<S> MsgData for MsgBuffer<S>
where
  S: Lease<str>,
{
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
  fn uri(&self) -> UriRef<'_> {
    self.uri.lease().to_ref()
  }
}

impl<S> MsgDataMut for MsgBuffer<S>
where
  S: Clear + Lease<str>,
{
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    &mut self.body
  }

  #[inline]
  fn clear(&mut self) {
    self.body.clear();
    self.headers.clear();
    self.uri.clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.body.clear();
    self.headers.clear();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    &mut self.headers
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (&mut self.body, &mut self.headers, self.uri.lease().to_ref())
  }
}

impl<S> Lease<[u8]> for MsgBuffer<S> {
  #[inline]
  fn lease(&self) -> &[u8] {
    &self.body
  }
}

impl<S> Lease<MsgBuffer<S>> for MsgBuffer<S> {
  #[inline]
  fn lease(&self) -> &MsgBuffer<S> {
    self
  }
}

impl<S> LeaseMut<MsgBuffer<S>> for MsgBuffer<S> {
  #[inline]
  fn lease_mut(&mut self) -> &mut MsgBuffer<S> {
    self
  }
}

impl<S> Debug for MsgBuffer<S>
where
  S: Lease<str>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("MsgBuffer")
      .field("body", &self.body)
      .field("headers", &self.headers)
      .field("uri", &self.uri)
      .finish()
  }
}

impl<S> Default for MsgBuffer<S>
where
  S: Default + Lease<str>,
{
  #[inline]
  fn default() -> Self {
    Self::from_uri(Uri::empty(S::default()))
  }
}

impl<S> From<Vector<u8>> for MsgBuffer<S>
where
  S: Default + Lease<str>,
{
  #[inline]
  fn from(from: Vector<u8>) -> Self {
    Self { body: from, headers: Headers::new(), uri: Uri::empty(S::default()) }
  }
}

#[cfg(feature = "std")]
impl<S> core::fmt::Write for MsgBuffer<S> {
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.body.extend_from_copyable_slice(s.as_bytes()).map_err(|_err| core::fmt::Error)
  }
}

#[cfg(feature = "std")]
impl<S> std::io::Write for MsgBuffer<S> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.body.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.body.flush()
  }
}
