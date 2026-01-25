use crate::{
  de::{Decode as _, Encode},
  misc::{LeaseMut, SuffixWriter},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    CurrEphemeralSecretKey, TlsBuffer, TlsConfig, TlsCrypto, TlsError, TlsMode, TlsModePlainText,
    TlsModeVerifyFull, TlsStream,
    cipher_suite::{CipherSuite, CipherSuiteTy},
    misc::fetch_rec_from_stream,
    protocol::{
      client_hello::ClientHello,
      handshake::{Handshake, HandshakeType},
      key_share_entry::KeyShareEntry,
      offered_psks::OfferedPsks,
      record::Record,
      record_content_type::RecordContentType,
      server_hello::ServerHello,
    },
  },
};

///
#[derive(Debug)]
pub struct TlsAcceptor<TB, TM> {
  //alpn_protocols: Vec<Vec<u8>>,
  //store: RootCertStore,
  tb: TB,
  tm: TM,
}

impl<TB, TM> TlsAcceptor<TB, TM>
where
  TB: LeaseMut<TlsBuffer>,
  TM: TlsMode,
{
  #[inline]
  pub async fn accept<RNG, S>(
    mut self,
    rng: &mut RNG,
    mut stream: S,
    mut tls_buffer: TB,
    tls_config: &TlsConfig<'_>,
  ) -> crate::Result<TlsStream<S, TB, TM, false>>
  where
    RNG: CryptoRng,
    S: Stream,
  {
    if TM::TY.is_plain_text() {
      return Ok(TlsStream::new(stream, self.tb, self.tm));
    }
    let client_hello = {
      let ty =
        fetch_rec_from_stream(&mut tls_buffer.lease_mut().network_buffer, &mut stream).await?;
      let RecordContentType::Handshake = ty else {
        return Err(TlsError::InvalidHandshake.into());
      };
      Handshake::<ClientHello<CurrEphemeralSecretKey, _>>::decode(
        &mut tls_buffer.lease_mut().network_buffer.current(),
      )?
    };
    {
      let record = Record::new(
        RecordContentType::Handshake,
        Handshake {
          data: ServerHello::new(
            seek_cipher_suite(
              &client_hello.data.tls_config().cipher_suites,
              &tls_config.inner.cipher_suites,
            )?,
            seek_keyshare(
              &client_hello.data.tls_config().key_shares,
              &tls_config.inner.key_shares,
            )?,
            client_hello.data.legacy_session_id().clone(),
            rng,
            seek_psk(&client_hello.data.tls_config().offered_psks, &[]),
          )?,
          msg_type: HandshakeType::ServerHello,
        },
      );
      let mut sw = SuffixWriter::new(0, &mut self.tb.lease_mut().write_buffer);
      record.encode(&mut sw)?;
      stream.write_all(sw.curr_bytes()).await?;
    }

    Ok(TlsStream::new(stream, self.tb, self.tm))
  }

  /// Skips all TLS actions nullifying all other TLS structures and configurations. Useful for
  /// testing.
  ///
  /// Shortcut of [`Self::set_tls_mode`] with [`TlsModePlainText`].
  pub fn set_plain_text(self) -> TlsAcceptor<TB, TlsModePlainText> {
    TlsAcceptor { tb: self.tb, tm: TlsModePlainText }
  }

  pub fn push_cert(self, cert: &[u8]) -> Self {
    self
  }

  pub fn push_priv_key(self, priv_key: &[u8]) -> Self {
    self
  }

  pub fn set_tls_mode<_TM>(self, tm: _TM) -> TlsAcceptor<TB, _TM> {
    TlsAcceptor { tb: self.tb, tm }
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

fn seek_cipher_suite(
  client: &[CipherSuiteTy],
  server: &[CipherSuiteTy],
) -> crate::Result<CipherSuiteTy> {
  for elem in server {
    if client.contains(elem) {
      return Ok(*elem);
    }
  }
  Err(TlsError::ServerNoCompatibleCypherSuite.into())
}

fn seek_keyshare<'client, 'rslt, 'server>(
  client: &[KeyShareEntry<'client>],
  server: &[KeyShareEntry<'server>],
) -> crate::Result<KeyShareEntry<'rslt>>
where
  'client: 'rslt,
  'server: 'rslt,
{
  for elem in server {
    if client.contains(elem) {
      return Ok(elem.clone());
    }
  }
  Err(TlsError::ServerNoCompatibleKeyShare.into())
}

fn seek_psk(offered: &OfferedPsks, stored: &[&[u8]]) -> Option<u16> {
  offered
    .offered_psks
    .iter()
    .position(|offered_psk| stored.contains(&offered_psk.identity.identity))?
    .try_into()
    .ok()
}
