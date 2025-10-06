use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use rustls_pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{
  TlsAcceptor, TlsConnector,
  client::TlsStream,
  rustls::{
    ClientConfig, ConfigBuilder, RootCertStore, ServerConfig, client::WantsClientCert,
    server::WantsServerCert,
  },
};

/// TLS client using `tokio-rustls` and associated crates.
#[derive(Debug)]
pub struct TokioRustlsConnector {
  alpn_protocols: Vec<Vec<u8>>,
  store: RootCertStore,
}

impl TokioRustlsConnector {
  /// From the automatic selection of dependencies.
  ///
  /// An error will be returned if no dependency that provides CA certificates is selected.
  #[inline]
  pub fn from_auto() -> crate::Result<Self> {
    #[cfg(feature = "webpki-roots")]
    {
      let mut this = Self::default();
      this.store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
      Ok(this)
    }
    #[cfg(not(feature = "webpki-roots"))]
    return Err(crate::Error::MissingCaProviders);
  }

  /// Connects using a generic stream without client authentication.
  #[inline]
  pub async fn connect_without_client_auth<S>(
    self,
    hostname: &str,
    stream: S,
  ) -> crate::Result<TlsStream<S>>
  where
    S: AsyncRead + AsyncWrite + Unpin,
  {
    Ok(
      self
        .tls_connector(|elem| elem.with_no_client_auth())
        .connect(Self::server_name(hostname)?, stream)
        .await?,
    )
  }

  /// Erases the current set of ALPN protocols and then pushes the expected ALPN value for a HTTP2
  /// connection.
  #[inline]
  pub fn http2(mut self) -> Self {
    self.alpn_protocols.clear();
    self.alpn_protocols.push("h2".into());
    self
  }

  /// Avoids additional round trips by specifying in advance which protocols should be used.
  #[inline]
  pub fn push_alpn_protocol(mut self, protocol: &[u8]) -> Self {
    self.alpn_protocols.push(protocol.into());
    self
  }

  /// Pushes a sequence of certificates.
  #[inline]
  pub fn push_certs(mut self, mut bytes: &[u8]) -> crate::Result<Self> {
    for rslt in rustls_pemfile::certs(&mut bytes) {
      let cert = rslt?;
      self.store.add(cert)?;
    }
    Ok(self)
  }

  fn server_name(hostname: &str) -> crate::Result<ServerName<'static>> {
    Ok(ServerName::try_from(String::from(hostname)).map_err(invalid_input_err)?)
  }

  fn tls_connector(
    self,
    cb: impl FnOnce(ConfigBuilder<ClientConfig, WantsClientCert>) -> ClientConfig,
  ) -> TlsConnector {
    let mut config = cb(ClientConfig::builder().with_root_certificates(self.store));
    config.alpn_protocols = self.alpn_protocols;
    TlsConnector::from(Arc::new(config))
  }
}

impl Default for TokioRustlsConnector {
  #[inline]
  fn default() -> Self {
    Self { alpn_protocols: Vec::new(), store: RootCertStore::empty() }
  }
}

/// TLS server using `tokio-rustls` and associated crates.
#[derive(Debug)]
pub struct TokioRustlsAcceptor {
  builder: ConfigBuilder<ServerConfig, WantsServerCert>,
  is_http2: bool,
}

impl TokioRustlsAcceptor {
  /// New instance without client authentication.
  #[inline]
  pub fn without_client_auth() -> Self {
    Self { builder: ServerConfig::builder().with_no_client_auth(), is_http2: false }
  }

  /// Creates a [`tokio_rustls::TlsAcceptor`] with a single certificate chain and matching private
  /// key.
  #[inline]
  pub fn build_with_cert_chain_and_priv_key(
    self,
    cert_chain: &[u8],
    priv_key: &[u8],
  ) -> crate::Result<TlsAcceptor> {
    let mut config = self.builder.with_single_cert(
      rustls_pemfile::certs(&mut &*cert_chain).collect::<Result<_, _>>()?,
      rustls_pemfile::private_key(&mut &*priv_key)?
        .ok_or_else(|| invalid_input_err("No private key"))?,
    )?;
    if self.is_http2 {
      config.alpn_protocols.clear();
      config.alpn_protocols.push("h2".into());
    }
    Ok(TlsAcceptor::from(Arc::new(config)))
  }

  /// Erases the set of ALPN protocols when building and then pushes the expected ALPN value for an
  /// HTTP2 connection.
  #[inline]
  pub const fn http2(mut self) -> Self {
    self.is_http2 = true;
    self
  }
}

fn invalid_input_err<E>(err: E) -> std::io::Error
where
  E: Into<Box<dyn core::error::Error + Send + Sync>>,
{
  std::io::Error::new(std::io::ErrorKind::InvalidInput, err)
}
