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

/// Basically performs the TLS handshake
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
    if TM::TY.is_plain_text() {
      return Ok(TlsStream::new(stream, self.tb, self.tm));
    }
    self.tb.lease_mut().clear();

    let mut key_schedule = if let Some(Psk { cipher_suite_ty, .. }) = psk {
      let mut key_schedule = KeySchedule::from_cipher_suite(CurrCipherSuite::from(cipher_suite_ty));
      key_schedule.early_secret(psk.map(|Psk { data, psk_ty, .. }| (data, psk_ty)))?;
      key_schedule
    } else {
      KeySchedule::from_cipher_suite(CurrCipherSuite::Aes128GcmSha256(<_>::default()))
    };
    let secrets = {
      let handshake = Handshake {
        data: ClientHello::<<CurrTlsCrypto as TlsCrypto>::EphemeralSecretKey, _>::new(
          rng,
          &tls_config.inner,
        )?,
        msg_type: HandshakeType::ClientHello,
      };
      let record = Record::new(RecordContentType::Handshake, &handshake);
      let mut sw = SuffixWriter::new(0, &mut self.tb.lease_mut().write_buffer);
      record.encode(&mut sw)?;
      stream.write_all(sw.curr_bytes()).await?;
      handshake.data.into_secrets()
    };
    {
      let ty = fetch_rec_from_stream(&mut self.tb.lease_mut().network_buffer, &mut stream).await?;
      let RecordContentType::Handshake = ty else {
        return Err(TlsError::InvalidHandshakeRecord.into());
      };
      let server_hello =
        Handshake::<ServerHello>::decode(&mut self.tb.lease_mut().network_buffer.current())?;
      let secret = secrets
        .into_iter()
        .find(|secret| secret.simplify() == server_hello.data.key_share().group)
        .ok_or(TlsError::SecretMismatch)?;
      if psk.is_none() {
        key_schedule.set_cipher_suite(CurrCipherSuite::from(server_hello.data.cipher_suite()));
        key_schedule.early_secret(None)?;
      }
      let shared_secret = secret.diffie_hellman(server_hello.data.key_share().opaque)?;
      key_schedule.handshake_secret(shared_secret.as_slice())?;
    }
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
