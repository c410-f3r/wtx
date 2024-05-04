use crate::{
  http::{Headers, Method, Version},
  misc::{Lease, Uri},
};

/// Shortcut for mutable referenced elements.
pub type RequestMut<'data, 'headers, 'uri, D> =
  Request<&'data mut D, &'headers mut Headers, &'headers str>;
/// Shortcut for referenced elements.
pub type RequestRef<'data, 'headers, 'uri, D> = Request<&'data D, &'headers Headers, &'headers str>;

/// An HTTP request received by a server or to be sent by a client.
#[derive(Debug)]
pub struct Request<D, H, U> {
  /// The payload of the request, which can be nothing.
  pub data: D,
  /// See [Headers].
  pub headers: H,
  /// See [Method].
  pub method: Method,
  /// See [Uri].
  pub uri: Uri<U>,
  /// See [Version].
  pub version: Version,
}

impl<D, H, U> Request<D, H, U>
where
  D: Lease<[u8]>,
  H: Lease<Headers>,
  U: Lease<str>,
{
  /// Constructor that defaults to an HTTP/2 version.
  #[inline]
  pub fn http2(data: D, headers: H, method: Method, uri: Uri<U>) -> Self {
    Self { data, headers, method, uri, version: Version::Http2 }
  }

  /// See [RequestRef].
  #[inline]
  pub fn to_ref(&self) -> RequestRef<'_, '_, '_, D> {
    RequestRef {
      data: &self.data,
      headers: self.headers.lease(),
      method: self.method,
      uri: self.uri.to_ref(),
      version: self.version,
    }
  }
}
