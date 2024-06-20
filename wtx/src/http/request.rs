use crate::{
  http::{Headers, Method, ReqResData, Version},
  misc::{Lease, Uri},
};

/// A [Request] with a URI composed by a string reference
pub type RequestStr<'uri, D> = Request<D, &'uri str>;

/// An HTTP request received by a server or to be sent by a client.
#[derive(Debug)]
pub struct Request<D, U> {
  /// See [`ReqResData`].
  pub data: D,
  /// See [`Method`].
  pub method: Method,
  /// See [`Uri`].
  pub uri: Uri<U>,
  /// See [`Version`].
  pub version: Version,
}

impl<D, U> Request<D, U>
where
  D: ReqResData,
  U: Lease<str>,
{
  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub fn http2(data: D, method: Method, uri: Uri<U>) -> Self {
    Self { data, method, uri, version: Version::Http2 }
  }

  /// Shortcut to access the body of `data`.
  #[inline]
  pub fn body(&self) -> &D::Body {
    self.data.body()
  }

  /// Shortcut to access the headers of `data`.
  #[inline]
  pub fn headers(&self) -> &Headers {
    self.data.headers()
  }
}
