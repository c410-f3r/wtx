use crate::{
  codec::{Decode as _, Encode as _},
  collections::ArrayVectorU8,
  crypto::SignatureTy,
  misc::{Lease, SingleTypeStorage, into_rslt},
  rng::CryptoRng,
  stream::{Stream, StreamReadItem},
  tls::{
    CipherSuite, HandshakePath, NamedGroup, TlsBuffer, TlsCertificateTy, TlsConfig, TlsError,
    TlsMode, TlsStream,
    key_schedule::KeySchedule,
    misc::fetch_rec_from_stream,
    protocol::{
      certificate::{Certificate, CertificateEntry},
      certificate_verify::CertificateVerify,
      client_hello::ClientHello,
      encrypted_extensions::EncryptedExtensions,
      finished::Finished,
      handshake::{Handshake, HandshakeType},
      key_share_entry::KeyShareEntry,
      offered_psks::OfferedPsks,
      record::Record,
      record_content_type::RecordContentType,
      server_hello::ServerHello,
    },
    read_record_info::ReadRecordInfo,
    tls_config::TlsConfigInner,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// TLS Acceptor
///
/// Performs TLS handshakes for servers.
#[derive(Debug)]
pub struct TlsAcceptor<RNG, S, TC> {
  buffer: TlsBuffer,
  config: TC,
  handshake_path: HandshakePath,
  key_schedule: KeySchedule,
  named_group: NamedGroup,
  rng: RNG,
  stream: S,
}

impl<RNG, S, TC, TM> TlsAcceptor<RNG, S, TC>
where
  TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
{
  /// The main parameters are provided by the user.
  #[inline]
  pub fn new(config: TC, rng: RNG, stream: S) -> Self {
    let named_group =
      config.lease().inner.named_groups.first().copied().unwrap_or(NamedGroup::default());
    Self {
      buffer: TlsBuffer::new(),
      config,
      handshake_path: HandshakePath::Full,
      key_schedule: KeySchedule::default(),
      named_group,
      rng,
      stream,
    }
  }

  /// The current handshake path. See [`HandshakePath`].
  #[inline]
  pub const fn handshake_path(&self) -> HandshakePath {
    self.handshake_path
  }

  /// The current signature algorithm. See [`NamedGroup`].
  #[inline]
  pub const fn named_group(&self) -> NamedGroup {
    self.named_group
  }

  /// Random Number Generator
  #[inline]
  pub const fn rng(&self) -> &RNG {
    &self.rng
  }

  /// Mutable version of [`Self::rng`].
  #[inline]
  pub const fn rng_mut(&mut self) -> &mut RNG {
    &mut self.rng
  }

  /// Changes the internal random number generator.
  #[inline]
  pub fn set_rng<_RNG>(self, value: _RNG) -> TlsAcceptor<_RNG, S, TC> {
    TlsAcceptor {
      buffer: self.buffer,
      config: self.config,
      handshake_path: self.handshake_path,
      key_schedule: self.key_schedule,
      named_group: self.named_group,
      rng: value,
      stream: self.stream,
    }
  }
}

impl<RNG, S, TC, TM> TlsAcceptor<RNG, S, TC>
where
  RNG: CryptoRng,
  S: Stream,
  TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
  TM: TlsMode,
{
  /// High level operation that automatically performs a full asynchronous handshake.
  ///
  /// Low level operations must not be mixed with high level operations.
  #[inline]
  pub async fn accept(mut self) -> crate::Result<StreamReadItem<TlsAcceptorRslt<RNG, S, TM>>>
  where
    RNG: CryptoRng,
    S: Stream,
  {
    if TM::TY.is_plain_text() {
      return Ok(StreamReadItem::from_item(TlsAcceptorRslt {
        handshake_path: self.handshake_path,
        named_group: self.named_group,
        rng: self.rng,
        stream: TlsStream::new(
          self.buffer,
          self.key_schedule,
          self.stream,
          self.config.lease().mode().clone(),
        ),
      }));
    }
    let Some(rri0) = self.fetch_rec_from_stream(false).await?.opt() else {
      return Ok(StreamReadItem::empty_cold());
    };
    self.manage_initial_client_record(&rri0)?;
    self.stream.write_all(&self.buffer.writer_buffer).await?;
    let Some(rri1) = self.fetch_rec_from_stream(false).await?.opt() else {
      return Ok(StreamReadItem::empty_cold());
    };
    self.manage_final_client_record(&rri1)?;
    Ok(StreamReadItem::from_item(TlsAcceptorRslt {
      handshake_path: self.handshake_path,
      named_group: self.named_group,
      rng: self.rng,
      stream: TlsStream::new(
        self.buffer,
        self.key_schedule,
        self.stream,
        self.config.lease().mode().clone(),
      ),
    }))
  }

  /// Low level operation responsible for processing data sent by clients. No other method should
  /// be called before it.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_initial_client_record(&mut self, rri: &ReadRecordInfo) -> crate::Result<()> {
    let RecordContentType::Handshake = rri.outer_ty else {
      return Err(TlsError::InvalidHandshake.into());
    };
    let client_hello = Handshake::<ClientHello<(), TlsConfigInner<_, TM>>>::decode(
      &mut TlsDecodeWrapper::from_bytes(self.buffer.reader_buffer.current()),
    )?;
    let client_hello_record = Record::new(
      RecordContentType::Handshake,
      Handshake {
        data: ServerHello::new(
          seek_cipher_suite(
            &client_hello.data.tls_config().cipher_suites,
            &self.config.lease().inner.cipher_suites,
          )?,
          false,
          seek_keyshare(
            &client_hello.data.tls_config().key_shares,
            &self.config.lease().inner.key_shares,
          )?,
          client_hello.data.legacy_session_id().clone(),
          &mut self.rng,
          seek_psk(&client_hello.data.tls_config().offered_psks, &[]),
        ),
        msg_type: HandshakeType::ServerHello,
      },
    );
    self.buffer.writer_buffer.clear();
    let mut ew = TlsEncodeWrapper::from_buffer(self.buffer.writer_buffer.suffix_pusher());
    client_hello_record.encode(&mut ew)?;
    let client_cert_type = first_compatible_cert_ty(
      &client_hello.data.tls_config().client_cert_types.0,
      &self.config.lease().inner.client_cert_types.0,
    )?;
    let server_cert_type = first_compatible_cert_ty(
      &client_hello.data.tls_config().server_cert_types.0,
      &self.config.lease().inner.server_cert_types.0,
    )?;

    let key_share = into_rslt(self.config.lease().inner.key_shares.first())?;
    let ee_record = Record::new(
      RecordContentType::ApplicationData,
      EncryptedExtensions::new(
        self.config.lease().inner.alpn.clone(),
        self.key_schedule.cipher_suite(),
        Some(client_cert_type),
        KeyShareEntry::new(key_share.group, &key_share.opaque),
        ArrayVectorU8::new(),
        &mut self.rng,
        Some(0),
        Some(server_cert_type),
      ),
    );
    ee_record.encode(&mut ew)?;

    let mut certificate_list = ArrayVectorU8::new();
    certificate_list.push(CertificateEntry::new(match client_cert_type {
      TlsCertificateTy::X509 => &self.config.lease().inner.public_key.0,
      TlsCertificateTy::RawPublicKey => &self.config.lease().inner.public_key.1,
    }))?;
    let certificate = Certificate::new(certificate_list, &[]);
    Record::new(RecordContentType::ApplicationData, certificate).encode(&mut ew)?;

    let certificate_verify = Record::new(
      RecordContentType::ApplicationData,
      CertificateVerify::new(SignatureTy::EcdsaSecp256r1Sha256, &[]),
    );
    certificate_verify.encode(&mut ew)?;

    let finished_record = Record::new(RecordContentType::ApplicationData, Finished::new(&[]));
    finished_record.encode(&mut ew)?;

    Ok(())
  }

  /// Low level operation responsible for processing data sent by clients. No other method should
  /// be called before it.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_final_client_record(&mut self, rri: &ReadRecordInfo) -> crate::Result<()> {
    let RecordContentType::ApplicationData = rri.outer_ty else {
      return Err(TlsError::InvalidHandshake.into());
    };
    let _finished =
      Finished::decode(&mut TlsDecodeWrapper::from_bytes(self.buffer.reader_buffer.current()))?;
    Ok(())
  }

  #[inline]
  async fn fetch_rec_from_stream(
    &mut self,
    decrypt: bool,
  ) -> Result<StreamReadItem<ReadRecordInfo>, crate::Error> {
    fetch_rec_from_stream(
      decrypt.then(|| self.key_schedule.read_mut().state_mut()),
      self.config.lease().max_fragment_length_actual(),
      &mut self.buffer.reader_buffer,
      &mut self.stream,
    )
    .await
  }
}

/// Returned by [`TlsAcceptor::accept`].
#[derive(Debug)]
pub struct TlsAcceptorRslt<RNG, S, TM> {
  /// See [`HandshakePath`].
  pub handshake_path: HandshakePath,
  /// See [`NamedGroup`].
  pub named_group: NamedGroup,
  /// Random Number Generator
  pub rng: RNG,
  /// See [`TlsStream`]
  pub stream: TlsStream<S, TM, false>,
}

fn first_compatible_cert_ty(
  client_hello: &[TlsCertificateTy],
  supported: &[TlsCertificateTy],
) -> crate::Result<TlsCertificateTy> {
  for lhs in supported {
    for rhs in client_hello {
      if lhs == rhs {
        return Ok(*lhs);
      }
    }
  }
  Err(TlsError::IncompatibleCertificateTypes.into())
}

fn seek_cipher_suite(client: &[CipherSuite], server: &[CipherSuite]) -> crate::Result<CipherSuite> {
  for elem in server {
    if client.contains(elem) {
      return Ok(*elem);
    }
  }
  Err(TlsError::ServerHasNoCompatibleCypherSuite.into())
}

fn seek_keyshare<'client, 'rslt, 'server, B>(
  client: &'client [KeyShareEntry<&'client [u8]>],
  server: &'server [KeyShareEntry<B>],
) -> crate::Result<KeyShareEntry<&'rslt [u8]>>
where
  B: Lease<[u8]>,
  'client: 'rslt,
  'server: 'rslt,
{
  for server_el in server {
    if let Some(elem) =
      client.iter().find(|client_el| client_el.opaque.lease() == server_el.opaque.lease())
    {
      return Ok(KeyShareEntry::new(elem.group, elem.opaque.lease()));
    }
  }
  Err(TlsError::ServerHasNoCompatibleKeyShare.into())
}

fn seek_psk<B>(offered: &OfferedPsks<B>, stored: &[&[u8]]) -> Option<u16>
where
  B: Lease<[u8]>,
{
  offered
    .offered_psks
    .iter()
    .position(|offered_psk| stored.contains(&offered_psk.identity.identity.lease()))?
    .try_into()
    .ok()
}
