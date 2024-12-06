use crate::{
  misc::{GenericTimeProvider, Stream},
  tls::{
    connector::TrustAnchor, item::Item, Connector, ConnectorBackend, Rustls, TlsStream,
    _invalid_input_err,
  },
};
use alloc::{string::String, sync::Arc};
use rustls::{client::UnbufferedClientConnection, version::TLS13, ClientConfig, RootCertStore};
use rustls_pki_types::{CertificateDer, Der, ServerName};

impl Connector<Rustls> {
  pub fn rustls() -> Self {
    Connector::new(Rustls)
  }
}

impl ConnectorBackend for Rustls {
  type Config = RootCertStore;
  type StreamAux = UnbufferedClientConnection;

  #[inline]
  fn config(&mut self) -> Self::Config {
    RootCertStore::empty()
  }

  #[inline]
  fn connect_without_client_auth<S>(
    self,
    config: Self::Config,
    hostname: &str,
    stream: S,
  ) -> crate::Result<TlsStream<Self::StreamAux, S, true>>
  where
    S: Stream,
  {
    #[cfg(feature = "aws-lc-rs")]
    let provider = rustls::crypto::aws_lc_rs::default_provider();
    #[cfg(all(feature = "ring", not(any(feature = "aws-lc-rs"))))]
    let provider = rustls::crypto::ring::default_provider();
    #[cfg(not(any(feature = "aws-lc-rs", feature = "ring")))]
    return Err(crate::tls::TlsError::MissingProvider.into());

    let time_provider = Arc::new(GenericTimeProvider);
    let client_config = ClientConfig::builder_with_details(Arc::new(provider), time_provider)
      .with_protocol_versions(&[&TLS13])?
      .with_root_certificates(config)
      .with_no_client_auth();
    let ucc = UnbufferedClientConnection::new(
      Arc::new(client_config),
      ServerName::try_from(String::from(hostname)).map_err(_invalid_input_err)?,
    )?;
    Ok(TlsStream::new(ucc, stream))
  }

  #[inline]
  fn extend_from_trust_anchors(
    &mut self,
    config: &mut Self::Config,
    trust_anchors: impl Iterator<Item = TrustAnchor<'static>>,
  ) -> crate::Result<()> {
    config.extend(trust_anchors.map(|ta| rustls_pki_types::TrustAnchor {
      subject: Der::from_slice(ta.subject),
      subject_public_key_info: Der::from_slice(ta.subject_public_key_info),
      name_constraints: ta.name_constraints.map(|el| Der::from_slice(el)),
    }));
    Ok(())
  }

  #[inline]
  fn push_der_cert(&mut self, config: &mut Self::Config, der_cert: &[u8]) -> crate::Result<()> {
    config.add(CertificateDer::from_slice(der_cert))?;
    Ok(())
  }

  #[inline]
  fn push_der_certs_from_pem(
    &mut self,
    config: &mut Self::Config,
    pem: &[u8],
  ) -> crate::Result<()> {
    for rslt in Item::rustls_pki_types(pem) {
      let Item::X509DerCertificate(der_cert) = rslt?;
      config.add(der_cert.into())?;
    }
    Ok(())
  }
}
