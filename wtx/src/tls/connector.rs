pub(crate) mod connector_backend;
#[cfg(feature = "rustls")]
mod rustls;

use crate::{
  misc::Stream,
  tls::{TlsStream, TrustAnchor},
};
use alloc::vec::Vec;

/// TLS client using `tokio-rustls` and associated crates.
#[derive(Debug)]
pub struct Connector<B>
where
  B: connector_backend::ConnectorBackend,
{
  alpn_protocols: Vec<Vec<u8>>,
  backend: B,
  backend_config: B::Config,
}

impl<B> Connector<B>
where
  B: connector_backend::ConnectorBackend,
{
  #[inline]
  pub fn new(mut backend: B) -> Self {
    let backend_config = backend.config();
    Self { alpn_protocols: Vec::new(), backend, backend_config }
  }

  /// From the automatic selection of dependencies.
  ///
  /// An error will be returned if no dependency that provides CA certificates is selected.
  #[inline]
  pub fn from_auto(backend: B) -> crate::Result<Self> {
    #[cfg(feature = "webpki-roots")]
    {
      let mut this = Self::new(backend);
      this.backend.extend_from_trust_anchors(
        &mut this.backend_config,
        webpki_roots::TLS_SERVER_ROOTS.iter().map(|el| TrustAnchor {
          name_constraints: el.name_constraints.as_ref().map(|der| der.as_ref()),
          subject: el.subject.as_ref(),
          subject_public_key_info: el.subject_public_key_info.as_ref(),
        }),
      )?;
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
  ) -> crate::Result<TlsStream<B::StreamAux, S, true>>
  where
    S: Stream,
  {
    self.backend.connect_without_client_auth(self.backend_config, hostname, stream)
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

  #[inline]
  pub fn push_der_cert(mut self, der_cert: &[u8]) -> crate::Result<Self> {
    self.backend.push_der_cert(&mut self.backend_config, der_cert)?;
    Ok(self)
  }

  #[inline]
  pub fn push_der_certs_from_pem(mut self, pem: &[u8]) -> crate::Result<Self> {
    self.backend.push_der_cert(&mut self.backend_config, pem)?;
    Ok(self)
  }
}
