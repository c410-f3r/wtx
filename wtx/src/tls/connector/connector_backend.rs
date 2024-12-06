use crate::{
  misc::Stream,
  tls::{connector::TrustAnchor, TlsStream},
};

/// TLS implementation responsable for initiating connections
pub trait ConnectorBackend {
  type Config;
  type StreamAux;

  fn config(&mut self) -> Self::Config;

  fn connect_without_client_auth<S>(
    self,
    config: Self::Config,
    hostname: &str,
    stream: S,
  ) -> crate::Result<TlsStream<Self::StreamAux, S, true>>
  where
    S: Stream;

  fn extend_from_trust_anchors(
    &mut self,
    config: &mut Self::Config,
    trust_anchors: impl Iterator<Item = TrustAnchor<'static>>,
  ) -> crate::Result<()>;

  fn push_der_cert(&mut self, config: &mut Self::Config, der_cert: &[u8]) -> crate::Result<()>;

  fn push_der_certs_from_pem(&mut self, config: &mut Self::Config, pem: &[u8])
    -> crate::Result<()>;
}

impl ConnectorBackend for () {
  type Config = ();
  type StreamAux = ();

  #[inline]
  fn config(&mut self) -> Self::Config {
    ()
  }

  #[inline]
  fn connect_without_client_auth<S>(
    self,
    _: Self::Config,
    _: &str,
    stream: S,
  ) -> crate::Result<TlsStream<Self::StreamAux, S, true>>
  where
    S: Stream,
  {
    Ok(TlsStream::new((), stream))
  }

  #[inline]
  fn extend_from_trust_anchors(
    &mut self,
    _: &mut Self::Config,
    _: impl Iterator<Item = TrustAnchor<'static>>,
  ) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  fn push_der_cert(&mut self, _: &mut Self::Config, _: &[u8]) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  fn push_der_certs_from_pem(&mut self, _: &mut Self::Config, _: &[u8]) -> crate::Result<()> {
    Ok(())
  }
}
