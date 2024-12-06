pub(crate) mod acceptor_backend;
#[cfg(feature = "rustls")]
mod rustls;

/// TLS implementation responsable for accepting connections
#[derive(Debug)]
pub struct Acceptor<B>
where
  B: acceptor_backend::AcceptorBackend,
{
  backend: B,
  backend_wca: B::WithoutClientAuth,
  is_http2: bool,
}

impl<B> Acceptor<B>
where
  B: acceptor_backend::AcceptorBackend,
{
  #[inline]
  pub fn without_client_auth(mut backend: B) -> Self {
    let backend_wca = backend.without_client_auth();
    Self { backend, backend_wca, is_http2: false }
  }

  /// Creates a [`tokio_rustls::TlsAcceptor`] with a single certificate chain and matching private
  /// key.
  #[inline]
  pub fn build_with_cert_chain_and_priv_key(
    self,
    cert_chain: &[u8],
    priv_key: &[u8],
  ) -> crate::Result<B::Acceptor> {
    self.backend.build_with_cert_chain_and_priv_key(
      self.backend_wca,
      cert_chain,
      self.is_http2,
      priv_key,
    )
  }

  /// Erases the set of ALPN protocols when building and then pushes the expected ALPN value for an
  /// HTTP2 connection.
  #[inline]
  pub fn http2(mut self) -> Self {
    self.is_http2 = true;
    self
  }
}
