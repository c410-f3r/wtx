use crate::{
  de::{Decode as _, Encode},
  misc::{LeaseMut, SuffixWriter},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    CurrCipherSuite, CurrTlsCrypto, Psk, TlsBuffer, TlsConfig, TlsCrypto, TlsError, TlsMode,
    TlsModePlainText, TlsModeVerifyFull, TlsStream,
    ephemeral_secret_key::EphemeralSecretKey,
    key_schedule::KeySchedule,
    misc::fetch_rec_from_stream,
    protocol::{
      client_hello::ClientHello,
      handshake::{Handshake, HandshakeType},
      record::Record,
      record_content_type::RecordContentType,
      server_hello::ServerHello,
    },
  },
};

/// High level abstractions that use [`TlsConnector`] under the hood.
#[derive(Debug)]
pub struct TlsStreamConnector<TB, TM> {
  //alpn_protocols: Vec<Vec<u8>>,
  //store: RootCertStore,
  tb: TB,
  tm: TM,
}

impl<TB, TM> TlsStreamConnector<TB, TM>
where
  TB: LeaseMut<TlsBuffer>,
  TM: TlsMode,
{
  #[inline]
  pub async fn connect<RNG, S>(
    mut self,
    psk: Option<Psk<'_>>,
    rng: &mut RNG,
    mut stream: S,
    tls_config: &TlsConfig<'_>,
  ) -> crate::Result<TlsStream<S, TB, TM, true>>
  where
    RNG: CryptoRng,
    S: Stream,
  {
    Ok(TlsStream::new(stream, self.tb, self.tm))
  }

  /// Skips all TLS actions nullifying all other TLS structures and configurations. Useful for
  /// testing.
  ///
  /// Shortcut of [`Self::set_tls_mode`] with [`TlsModePlainText`].
  pub fn set_plain_text(self) -> TlsStreamConnector<TB, TlsModePlainText> {
    TlsStreamConnector { tb: self.tb, tm: TlsModePlainText }
  }

  pub fn push_cert(self, cert: &[u8]) -> Self {
    self
  }

  pub fn set_tls_mode<_TM>(self, tm: _TM) -> TlsStreamConnector<TB, _TM> {
    TlsStreamConnector { tb: self.tb, tm }
  }
}

impl TlsStreamConnector<TlsBuffer, TlsModeVerifyFull> {
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

impl Default for TlsStreamConnector<TlsBuffer, TlsModeVerifyFull> {
  #[inline]
  fn default() -> Self {
    Self { tb: TlsBuffer::default(), tm: TlsModeVerifyFull }
  }
}
