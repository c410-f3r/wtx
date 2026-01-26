use crate::{
  collection::ArrayVectorU8,
  de::{Decode as _, Encode},
  misc::{LeaseMut, SuffixWriter},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    CurrCipherSuite, CurrEphemeralSecretKey, MAX_KEY_SHARES_LEN, Psk, TlsBuffer, TlsConfig,
    TlsError, TlsMode, TlsModeVerifyFull, TlsStream,
    ephemeral_secret_key::EphemeralSecretKey,
    key_schedule::KeySchedule,
    misc::fetch_rec_from_stream,
    protocol::{
      alert::Alert,
      client_hello::ClientHello,
      encrypted_extensions::EncryptedExtensions,
      handshake::{Handshake, HandshakeType},
      record::Record,
      record_content_type::RecordContentType,
      server_hello::ServerHello,
    },
  },
};

/// Basically performs the TLS handshake
#[derive(Debug)]
pub struct TlsConnector<TB, TM> {
  //alpn_protocols: Vec<Vec<u8>>,
  //store: RootCertStore,
  has_psk: bool,
  key_schedule: KeySchedule<CurrCipherSuite>,
  tb: TB,
  tm: TM,
}

impl<TB, TM> TlsConnector<TB, TM>
where
  TB: LeaseMut<TlsBuffer>,
  TM: TlsMode,
{
  /// High level operation that automatically performs a full handshake.
  ///
  /// Low level operations must not be mixed with high level operations.
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
    let secrets = self.write_client_hello(psk, rng, tls_config)?;
    stream.write_all(&self.tb.lease_mut().write_buffer).await?;
    let ty = fetch_rec_from_stream(&mut self.tb.lease_mut().network_buffer, &mut stream).await?;
    if !self.manage_initial_server_record(secrets, ty)? {
      stream.write_all(&self.tb.lease_mut().write_buffer).await?;
      return Err(TlsError::AbortedHandshake.into());
    }
    Ok(TlsStream::new(stream, self.tb, self.tm))
  }

  /// Low level operation that must be called after [`Self::write_client_hello`].
  ///
  /// Returns `false` if the connection was aborted by the server.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_initial_server_record(
    &mut self,
    secrets: ArrayVectorU8<CurrEphemeralSecretKey, MAX_KEY_SHARES_LEN>,
    ty: RecordContentType,
  ) -> crate::Result<bool> {
    match ty {
      RecordContentType::Handshake => {}
      RecordContentType::Alert => {
        self.read_and_write_alert();
        return Ok(false);
      }
      _ => return Err(TlsError::InvalidHandshake.into()),
    }
    let server_hello =
      Handshake::<ServerHello>::decode(&mut self.tb.lease_mut().network_buffer.current())?;
    let secret = secrets
      .into_iter()
      .find(|secret| secret.simplify() == server_hello.data.key_share().group)
      .ok_or(TlsError::SecretMismatch)?;
    if !self.has_psk {
      self.key_schedule.set_cipher_suite(CurrCipherSuite::from(server_hello.data.cipher_suite()));
      self.key_schedule.early_secret(None)?;
    }
    let shared_secret = secret.diffie_hellman(server_hello.data.key_share().opaque)?;
    self.key_schedule.handshake_secret(shared_secret.as_slice())?;
    Ok(true)
  }

  /// Low level operation that must be called after [`Self::write_client_hello`].
  ///
  /// Returns `None` if the connection was aborted by the server or `Some(false)` if
  /// this method needs to be called again.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_remaining_server_records<RNG>(
    &mut self,
    ty: RecordContentType,
  ) -> crate::Result<Option<bool>> {
    match ty {
      RecordContentType::Handshake => {}
      RecordContentType::Alert => {
        self.read_and_write_alert();
        return Ok(None);
      }
      _ => return Err(TlsError::InvalidHandshake.into()),
    }
    let mut hs = Handshake::<&[u8]>::decode(&mut self.tb.lease_mut().network_buffer.current())?;
    match hs.msg_type {
      HandshakeType::EncryptedExtensions => {
        let _encrypted_extensions = EncryptedExtensions::decode(&mut hs.data)?;
      }
      HandshakeType::Certificate => {}
      HandshakeType::CertificateRequest => {}
      HandshakeType::CertificateVerify => {}
      HandshakeType::Finished => return Ok(Some(true)),
      _ => {
        return Err(TlsError::InvalidHandshake.into());
      }
    }
    Ok(Some(false))
  }

  /// Low level operation responsible for informing the local parameters to the remote server. No other method should
  /// be called before it.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn write_client_hello<RNG>(
    &mut self,
    psk: Option<Psk<'_>>,
    rng: &mut RNG,
    tls_config: &TlsConfig<'_>,
  ) -> crate::Result<ArrayVectorU8<CurrEphemeralSecretKey, MAX_KEY_SHARES_LEN>>
  where
    RNG: CryptoRng,
  {
    if TM::TY.is_plain_text() {
      return Ok(ArrayVectorU8::new());
    }
    self.tb.lease_mut().clear();
    if let Some(Psk { cipher_suite_ty, .. }) = psk {
      let mut key_schedule = KeySchedule::from_cipher_suite(CurrCipherSuite::from(cipher_suite_ty));
      key_schedule.early_secret(psk.map(|Psk { data, psk_ty, .. }| (data, psk_ty)))?;
      self.key_schedule = key_schedule;
      self.has_psk = true;
    }
    let mut secrets = ArrayVectorU8::new();
    for key_share in &tls_config.inner.key_shares {
      secrets.push(CurrEphemeralSecretKey::random(key_share.group, rng)?)?;
    }
    let handshake = Handshake {
      data: ClientHello::new(rng, &secrets, &tls_config.inner)?,
      msg_type: HandshakeType::ClientHello,
    };
    let record = Record::new(RecordContentType::Handshake, &handshake);
    self.tb.lease_mut().write_buffer.clear();
    let mut sw = SuffixWriter::new(0, &mut self.tb.lease_mut().write_buffer);
    record.encode(&mut sw)?;
    Ok(secrets)
  }

  /// Low level operation that must be called after [`Self::manage_remaining_server_records`] is concluded.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn write_final_records<RNG, S>() {}

  fn read_and_write_alert(&mut self) -> crate::Result<()> {
    let alert = Alert::decode(&mut self.tb.lease_mut().network_buffer.current())?;
    self.tb.lease_mut().write_buffer.clear();
    let mut sw = SuffixWriter::new(0, &mut self.tb.lease_mut().write_buffer);
    Record::new(RecordContentType::Alert, alert).encode(&mut sw)?;
    Ok(())
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
    let dummy_cipher_suite = CurrCipherSuite::Aes128GcmSha256(<_>::default());
    Self {
      has_psk: false,
      key_schedule: KeySchedule::from_cipher_suite(dummy_cipher_suite),
      tb: TlsBuffer::default(),
      tm: TlsModeVerifyFull,
    }
  }
}
