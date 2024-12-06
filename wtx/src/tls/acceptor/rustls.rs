use crate::tls::{Acceptor, AcceptorBackend, Rustls, _invalid_input_err};
use alloc::sync::Arc;
use rustls::{server::WantsServerCert, ConfigBuilder, ServerConfig};
use tokio_rustls::TlsAcceptor;

impl Acceptor<Rustls> {
  pub fn without_client_auth_rustls() -> Self {
    Self::without_client_auth(Rustls)
  }
}

impl AcceptorBackend for Rustls {
  type Acceptor = TlsAcceptor;
  type WithoutClientAuth = ConfigBuilder<ServerConfig, WantsServerCert>;

  #[inline]
  fn without_client_auth(&mut self) -> Self::WithoutClientAuth {
    ServerConfig::builder().with_no_client_auth()
  }

  #[inline]
  fn build_with_cert_chain_and_priv_key(
    self,
    wca: Self::WithoutClientAuth,
    cert_chain: &[u8],
    is_http2: bool,
    priv_key: &[u8],
  ) -> crate::Result<Self::Acceptor> {
    let mut config = wca.with_single_cert(
      rustls_pemfile::certs(&mut &*cert_chain).collect::<Result<_, _>>()?,
      rustls_pemfile::private_key(&mut &*priv_key)?.ok_or_else(|| _invalid_input_err("No key"))?,
    )?;
    if is_http2 {
      config.alpn_protocols.clear();
      config.alpn_protocols.push("h2".into());
    }
    Ok(TlsAcceptor::from(Arc::new(config)))
  }
}
