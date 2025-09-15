use crate::misc::Lease;

/// Transport Layer Security
pub trait StreamWithTls {
  /// Channel binding data defined in [RFC 5929].
  ///
  /// [RFC 5929]: https://tools.ietf.org/html/rfc5929
  type TlsServerEndPoint: Lease<[u8]>;

  /// See `Self::TlsServerEndPoint`.
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>>;
}

impl StreamWithTls for () {
  type TlsServerEndPoint = [u8; 0];

  #[inline]
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>> {
    Ok(None)
  }
}

impl<T> StreamWithTls for &T
where
  T: StreamWithTls,
{
  type TlsServerEndPoint = T::TlsServerEndPoint;

  #[inline]
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>> {
    (*self).tls_server_end_point()
  }
}
