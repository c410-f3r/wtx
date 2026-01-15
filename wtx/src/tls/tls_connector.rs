use crate::{
  de::{Decode as _, Encode},
  misc::{LeaseMut, SuffixWriter},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    CurrTlsCrypto, NamedGroup, TlsBuffer, TlsConfig, TlsCrypto, TlsError, TlsMode,
    TlsModePlainText, TlsModeVerifyFull, TlsStream,
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

///
#[derive(Debug)]
pub struct TlsConnector<TB, TM> {
  //alpn_protocols: Vec<Vec<u8>>,
  //store: RootCertStore,
  tb: TB,
  tm: TM,
}

impl<TB, TM> TlsConnector<TB, TM>
where
  TB: LeaseMut<TlsBuffer>,
  TM: TlsMode,
{
  #[inline]
  pub async fn connect<RNG, S>(
    mut self,
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

      let shared_secret = match server_hello.data.key_share().group {
        NamedGroup::Secp256r1 => todo!(),
        NamedGroup::Secp384r1 => todo!(),
        NamedGroup::X25519 => todo!(),
      };
    }
    Ok(TlsStream::new(stream, self.tb, self.tm))
  }

  /// Skips all TLS actions nullifying all other TLS structures and configurations. Useful for
  /// testing.
  ///
  /// Shortcut of [`Self::set_tls_mode`] with [`TlsModePlainText`].
  pub fn set_plain_text(self) -> TlsConnector<TB, TlsModePlainText> {
    TlsConnector { tb: self.tb, tm: TlsModePlainText }
  }

  pub fn push_cert(self, cert: &[u8]) -> Self {
    self
  }

  pub fn set_tls_mode<_TM>(self, tm: _TM) -> TlsConnector<TB, _TM> {
    TlsConnector { tb: self.tb, tm }
  }
}

impl TlsConnector<TlsBuffer, TlsModeVerifyFull> {
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

impl Default for TlsConnector<TlsBuffer, TlsModeVerifyFull> {
  #[inline]
  fn default() -> Self {
    Self { tb: TlsBuffer::default(), tm: TlsModeVerifyFull }
  }
}
