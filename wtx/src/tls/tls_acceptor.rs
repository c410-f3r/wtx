use crate::tls::{TlsBuffer, TlsConfig, TlsModePlainText, TlsModeVerifyFull, TlsStream};

///
#[derive(Debug)]
pub struct TlsAcceptor<TB, TM> {
  //alpn_protocols: Vec<Vec<u8>>,
  //store: RootCertStore,
  tb: TB,
  tm: TM,
}

impl<TB, TM> TlsAcceptor<TB, TM> {
  #[inline]
  pub async fn accept<S>(
    self,
    _: &TlsConfig<'_>,
    stream: S,
  ) -> crate::Result<TlsStream<S, TB, TM, false>> {
    Ok(TlsStream::new(stream, self.tb, self.tm))
  }

  // Useful for testing
  pub fn plain_text(self) -> TlsAcceptor<TB, TlsModePlainText> {
    TlsAcceptor { tb: self.tb, tm: TlsModePlainText }
  }

  pub fn push_cert(self, cert: &[u8]) -> Self {
    self
  }
}

impl TlsAcceptor<TlsBuffer, TlsModeVerifyFull> {
  /// From the automatic selection of dependencies.
  ///
  /// An error will be returned if no dependency that provides CA certificates is selected.
  #[inline]
  pub fn from_auto_ca_providers() -> crate::Result<Self> {
    #[cfg(feature = "webpki-roots")]
    {
      let mut this = Self::default();
      //this.store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
      Ok(this)
    }
    #[cfg(not(feature = "webpki-roots"))]
    return Err(crate::Error::MissingCaProviders);
  }
}

impl Default for TlsAcceptor<TlsBuffer, TlsModeVerifyFull> {
  #[inline]
  fn default() -> Self {
    Self { tb: TlsBuffer::default(), tm: TlsModeVerifyFull }
  }
}
