use alloc::{boxed::Box, string::String, vec::Vec};
use rustls_pki_types::ServerName;
use std::sync::Arc;
use tokio::{
  io::{AsyncRead, AsyncWrite},
  net::TcpStream,
};
use tokio_rustls::{
  client::TlsStream,
  rustls::{server::WantsServerCert, ClientConfig, ConfigBuilder, RootCertStore, ServerConfig},
  TlsConnector,
};

/// TLS client using `tokio-rustls` and associated crates.
#[derive(Debug)]
pub struct TokioRustlsConnector {
  _alpn_protocols: Vec<Vec<u8>>,
  _store: RootCertStore,
}

impl TokioRustlsConnector {
  /// From the certificates of the `webpkis-roots` project.
  ///
  /// Defaults to an ALPN suitable for HTTP/2 connections.
  #[cfg(feature = "webpki-roots")]
  pub fn from_webpki_roots() -> Self {
    let mut this = Self::default();
    this._store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    this._http2()
  }

  /// Avoids additional round trips by specifying in advance which protocols should be used.
  pub fn push_alpn_protocol(mut self, protocol: &[u8]) -> Self {
    self._alpn_protocols.push(protocol.into());
    self
  }

  /// Pushes a sequence of certificates.
  #[cfg(feature = "rustls-pemfile")]
  pub fn push_certs(mut self, bytes: &[u8]) -> crate::Result<Self> {
    for rslt in rustls_pemfile::certs(&mut &*bytes) {
      let cert = rslt?;
      self._store.add(cert)?;
    }
    Ok(self)
  }

  /// Connects using a generic stream.
  pub async fn with_generic_stream<S>(
    self,
    hostname: &str,
    stream: S,
  ) -> crate::Result<TlsStream<S>>
  where
    S: AsyncRead + AsyncWrite + Unpin,
  {
    Ok(self.tls_connector().connect(Self::server_name(hostname)?, stream).await?)
  }

  /// Connects using a [TcpStream] stream.
  pub async fn with_tcp_stream(
    self,
    host: &str,
    hostname: &str,
  ) -> crate::Result<TlsStream<TcpStream>> {
    let stream = TcpStream::connect(host).await?;
    Ok(self.tls_connector().connect(Self::server_name(hostname)?, stream).await?)
  }

  /// Erases the current set of ALPN protocols and then pushes the expected ALPN value for a HTTP2
  /// connection.
  fn _http2(mut self) -> Self {
    self._alpn_protocols.clear();
    self._alpn_protocols.push("h2".into());
    self
  }

  fn server_name(hostname: &str) -> crate::Result<ServerName<'static>> {
    Ok(ServerName::try_from(String::from(hostname)).map_err(invalid_input_err)?)
  }

  fn tls_connector(self) -> TlsConnector {
    let mut config =
      ClientConfig::builder().with_root_certificates(self._store).with_no_client_auth();
    config.alpn_protocols = self._alpn_protocols;
    TlsConnector::from(Arc::new(config))
  }
}

impl Default for TokioRustlsConnector {
  #[inline]
  fn default() -> Self {
    Self { _alpn_protocols: Vec::new(), _store: RootCertStore::empty() }
  }
}

/// TLS server using `tokio-rustls` and associated crates.
#[derive(Debug)]
pub struct TokioRustlsAcceptor {
  _builder: ConfigBuilder<ServerConfig, WantsServerCert>,
  _is_http2: bool,
}

impl TokioRustlsAcceptor {
  /// Erases the set of ALPN protocols when building and then pushes the expected ALPN value for a
  /// HTTP2 connection.
  pub fn http2(mut self) -> Self {
    self._is_http2 = true;
    self
  }

  /// Sets a single certificate chain and matching private key.
  #[cfg(feature = "rustls-pemfile")]
  pub fn with_cert_chain_and_priv_key(
    self,
    cert_chain: &[u8],
    priv_key: &[u8],
  ) -> crate::Result<tokio_rustls::TlsAcceptor> {
    let mut config = self._builder.with_single_cert(
      rustls_pemfile::certs(&mut &*cert_chain).collect::<Result<_, _>>()?,
      rustls_pemfile::private_key(&mut &*priv_key)?
        .ok_or_else(|| invalid_input_err("No private key found"))?,
    )?;
    if self._is_http2 {
      config.alpn_protocols.clear();
      config.alpn_protocols.push("h2".into());
    }
    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(config)))
  }
}

impl Default for TokioRustlsAcceptor {
  #[inline]
  fn default() -> Self {
    Self { _builder: ServerConfig::builder().with_no_client_auth(), _is_http2: false }
  }
}

fn invalid_input_err<E>(err: E) -> std::io::Error
where
  E: Into<Box<dyn core::error::Error + Send + Sync>>,
{
  std::io::Error::new(std::io::ErrorKind::InvalidInput, err)
}
