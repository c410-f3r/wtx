use crate::{
  codec::{Decode as _, Encode as _},
  collections::ArrayVectorU8,
  crypto::SignatureTy,
  misc::{Lease, SingleTypeStorage},
  rng::CryptoRng,
  stream::{Stream, StreamReadItem},
  sync::{Arc, SyncMutex},
  tls::{
    CipherSuite, DLFT_MAX_FRAGMENT_LENGTH, HandshakePath, MaxFragmentLength, NamedGroup, Psks,
    TlsBuffer, TlsCertificateTy, TlsConfig, TlsError, TlsMode, TlsStream,
    key_schedule::KeySchedule,
    misc::{fetch_rec_from_stream, server_sig_msg, write_data},
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
    tls_hash::TlsHash,
  },
};
use hashbrown::HashMap;

/// TLS Acceptor
///
/// Performs TLS handshakes for servers.
#[derive(Debug)]
pub struct TlsAcceptor<RNG, S, TC> {
  buffer: TlsBuffer,
  config: TC,
  handshake_path: HandshakePath,
  key_schedule: KeySchedule,
  max_fragment_length: u16,
  named_group: NamedGroup,
  psks: Psks,
  rng: RNG,
  stream: S,
  transcript_hash: TlsHash,
}

impl<RNG, S, TC, TM> TlsAcceptor<RNG, S, TC>
where
  TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
{
  /// The main parameters are provided by the user.
  #[inline]
  pub fn new(config: TC, rng: RNG, stream: S) -> Self {
    let cfg_ref = config.lease();
    let key_schedule = KeySchedule::default();
    let transcript_hash = key_schedule.cipher_suite().hash_new();
    let max_fragment_length =
      cfg_ref.max_fragment_length().map_or(DLFT_MAX_FRAGMENT_LENGTH, |el| el.num());
    let named_group = cfg_ref.inner.named_groups.first().copied().unwrap_or(NamedGroup::default());
    Self {
      buffer: TlsBuffer::new(),
      config,
      handshake_path: HandshakePath::Full,
      key_schedule,
      max_fragment_length,
      named_group,
      psks: Arc::new(SyncMutex::new(HashMap::new())),
      rng,
      stream,
      transcript_hash,
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

  /// Pre Shares Keys
  #[inline]
  pub const fn psks(&self) -> &Psks {
    &self.psks
  }

  /// Mutable version of [`Self::psks`].
  #[inline]
  pub const fn psks_mut(&mut self) -> &mut Psks {
    &mut self.psks
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
  pub async fn accept(mut self) -> crate::Result<StreamReadItem<TlsAcceptorRslt<RNG, S, TM>>> {
    if TM::TY.is_plain_text() {
      return Ok(StreamReadItem::from_item(TlsAcceptorRslt {
        handshake_path: self.handshake_path,
        named_group: self.named_group,
        rng: self.rng,
        stream: TlsStream::new(
          self.buffer,
          self.key_schedule,
          self.max_fragment_length,
          self.stream,
          self.config.lease().mode().clone(),
        ),
      }));
    }
    let Some(rri0) = self.fetch_rec_from_stream(false).await?.opt() else {
      return Ok(StreamReadItem::empty_cold());
    };
    let indices = self.manage_initial_client_record(&rri0)?;
    let buffer = self.buffer.reader_buffer.buffer_mut();
    write_data(
      &[
        buffer.get(indices[0]..indices[1]).unwrap_or_default(),
        buffer.get(indices[1]..indices[2]).unwrap_or_default(),
        buffer.get(indices[2]..indices[3]).unwrap_or_default(),
        buffer.get(indices[3]..).unwrap_or_default(),
      ],
      self.key_schedule.write_mut(),
      self.max_fragment_length,
      &mut self.stream,
      &mut self.buffer.writer_buffer,
    )
    .await?;
    buffer.truncate(indices[0]);
    let Some(rri1) = self.fetch_rec_from_stream(true).await?.opt() else {
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
        self.max_fragment_length,
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
  pub fn manage_initial_client_record(
    &mut self,
    rri: &ReadRecordInfo,
  ) -> crate::Result<[usize; 4]> {
    let RecordContentType::Handshake = rri.outer_ty else {
      return Err(TlsError::InvalidHandshake.into());
    };
    let (client_cert_type, max_fragment_length, server_cert_type, sig_ty) = self.negotiate()?;
    self.buffer.reader_buffer.clear_if_exhausted();
    let reader_buffer = self.buffer.reader_buffer.buffer_mut();
    let mut curr_idx = reader_buffer.len();
    let mut indices = [0usize; 4];
    let encrypted_extensions = EncryptedExtensions::new(
      self.config.lease().inner.alpn.clone(),
      self.key_schedule.cipher_suite(),
      Some(client_cert_type),
      ArrayVectorU8::new(),
      max_fragment_length,
      &mut self.rng,
      Some(0),
      Some(server_cert_type),
    );
    encrypted_extensions.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
    self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());
    indices[0] = curr_idx;
    curr_idx = reader_buffer.len();
    indices[1] = curr_idx;

    let mut certificate_list = ArrayVectorU8::new();
    certificate_list.push(CertificateEntry::new(match client_cert_type {
      TlsCertificateTy::X509 => &self.config.lease().inner.public_key.x509,
      TlsCertificateTy::RawPublicKey => &self.config.lease().inner.public_key.raw_public_key,
    }))?;
    let certificate = Certificate::new(certificate_list, &[]);
    certificate.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
    self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());
    curr_idx = reader_buffer.len();
    indices[2] = curr_idx;

    let mut sign_key = sig_ty.sign_key_from_pkcs8(&self.config.lease().inner.secret_key)?;
    let msg = server_sig_msg(self.transcript_hash.clone().finalize().lease())?;
    let signature = sign_key.sign(&mut self.rng, &msg)?;
    let certificate_verify = CertificateVerify::new(sig_ty, signature.as_ref());
    certificate_verify.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
    self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());
    curr_idx = reader_buffer.len();
    indices[3] = curr_idx;

    let verify_data = self
      .key_schedule
      .write_mut()
      .state_mut()
      .create_finished_verify_data(self.transcript_hash.clone().finalize().lease())?;
    let finished = Finished::new(verify_data.as_slice());
    finished.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
    self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());

    Ok(indices)
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
    let current = self.buffer.reader_buffer.current();
    let plaintext = current.get(..rri.plaintext_len).unwrap_or_default();
    let mut remote_dw = TlsDecodeWrapper::from_bytes(plaintext);
    let hs = Handshake::<&[u8]>::decode(&mut remote_dw)?;
    if hs.msg_type != HandshakeType::Finished {
      return Err(TlsError::InvalidHandshake.into());
    }
    *remote_dw.bytes_mut() = hs.data;
    let finished = Finished::decode(&mut remote_dw)?;
    self.key_schedule.read_mut().state_mut().verify_finished_record(
      self.transcript_hash.clone().finalize().lease(),
      finished.verify_data(),
    )?;
    self.transcript_hash.update(plaintext);
    self.key_schedule.master_secret(&self.transcript_hash.clone().finalize())?;
    Ok(())
  }

  #[inline]
  async fn fetch_rec_from_stream(
    &mut self,
    decrypt: bool,
  ) -> crate::Result<StreamReadItem<ReadRecordInfo>> {
    fetch_rec_from_stream(
      decrypt.then(|| self.key_schedule.read_mut().state_mut()),
      self.max_fragment_length,
      &mut self.buffer.reader_buffer,
      &mut self.stream,
    )
    .await
  }

  #[inline]
  fn negotiate(
    &mut self,
  ) -> crate::Result<(TlsCertificateTy, Option<MaxFragmentLength>, TlsCertificateTy, SignatureTy)>
  where
    TM: TlsMode,
  {
    let client_hello_bytes = self.buffer.reader_buffer.current();
    let client_hello = Handshake::<ClientHello<(), TlsConfigInner<_, TM>>>::decode(
      &mut TlsDecodeWrapper::from_bytes(client_hello_bytes),
    )?;
    let cipher_suite = seek_cipher_suite(
      &client_hello.data.tls_config().cipher_suites,
      &self.config.lease().inner.cipher_suites,
    )?;
    self.key_schedule.set_cipher_suite(cipher_suite);
    self.transcript_hash = cipher_suite.hash_new();
    self.transcript_hash.update(client_hello_bytes);
    let client_cert_type = seek_cert_ty(
      &client_hello.data.tls_config().client_cert_types.0,
      &self.config.lease().inner.client_cert_types.0,
    )?;
    let (client_opaque, server_kse) = seek_key_share(
      &client_hello.data.tls_config().key_shares,
      &self.config.lease().inner.key_shares,
    )?;
    let server_cert_type = seek_cert_ty(
      &client_hello.data.tls_config().server_cert_types.0,
      &self.config.lease().inner.server_cert_types.0,
    )?;
    self.named_group = server_kse.group;
    let max_fragment_length = client_hello.data.tls_config().max_fragment_length;
    if let Some(elem) = max_fragment_length {
      self.max_fragment_length = elem.num();
    }
    let sig_ty = seek_signature_algorithm_cert(
      &client_hello.data.tls_config().signature_algorithms_cert,
      &self.config.lease().inner.signature_algorithms_cert,
    )?;
    let legacy_session_id = *client_hello.data.legacy_session_id();
    let offered_psks = &client_hello.data.tls_config().offered_psks;
    let binders_len: usize = offered_psks
      .offered_psks
      .iter()
      .map(|element| element.binder.lease().len().wrapping_add(1))
      .sum();
    let truncated_len = client_hello_bytes.len().saturating_sub(binders_len);
    let selected_identity = seek_psk(
      client_hello_bytes.get(..truncated_len).unwrap_or_default(),
      &mut self.handshake_path,
      &mut self.key_schedule,
      offered_psks,
      &self.psks,
    )?;
    let agreement = server_kse.group.agreement(&mut self.rng)?;
    let ephemeral_pk = agreement.public_key()?;
    let secret = agreement.diffie_hellman(client_opaque)?;
    let server_hello = Handshake {
      data: ServerHello::new(
        cipher_suite,
        false,
        KeyShareEntry::new(server_kse.group, ephemeral_pk.as_ref()),
        legacy_session_id,
        &mut self.rng,
        selected_identity,
      ),
      msg_type: HandshakeType::ServerHello,
    };
    let writer_buffer = &mut self.buffer.writer_buffer;
    let server_hello_rec = Record::new(RecordContentType::Handshake, server_hello);
    writer_buffer.clear();
    server_hello_rec.encode(&mut TlsEncodeWrapper::from_buffer(writer_buffer))?;
    self.transcript_hash.update(writer_buffer.get(5..).unwrap_or_default());
    self
      .key_schedule
      .handshake_secret(secret.as_ref(), &self.transcript_hash.clone().finalize())?;
    Ok((client_cert_type, max_fragment_length, server_cert_type, sig_ty))
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

fn seek_cert_ty(
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

fn seek_key_share<'client, 'rslt, 'server, B>(
  client: &'client [KeyShareEntry<&'client [u8]>],
  server: &'server [KeyShareEntry<B>],
) -> crate::Result<(&'rslt [u8], KeyShareEntry<&'rslt [u8]>)>
where
  B: Lease<[u8]>,
  'client: 'rslt,
  'server: 'rslt,
{
  for server_el in server {
    let Some(client_el) = client.iter().find(|client_el| client_el.group == server_el.group) else {
      continue;
    };
    return Ok((client_el.opaque, KeyShareEntry::new(server_el.group, server_el.opaque.lease())));
  }
  Err(TlsError::ServerHasNoCompatibleKeyShare.into())
}

fn seek_psk<B>(
  client_hello_bytes: &[u8],
  handshake_path: &mut HandshakePath,
  key_schedule: &mut KeySchedule,
  offered: &OfferedPsks<B>,
  stored: &Psks,
) -> crate::Result<Option<u16>>
where
  B: Lease<[u8]>,
{
  if offered.offered_psks.is_empty() {
    return Ok(None);
  }
  let cipher_suite = key_schedule.cipher_suite();
  let hash = cipher_suite.hash_digest([client_hello_bytes]);
  let lock = stored.lock();
  let mut idx: u16 = 0;
  for offered_psk in &offered.offered_psks {
    if let Some(psk) = lock.get(offered_psk.identity.identity.lease())
      && psk.cipher_suite == cipher_suite
      && let local_hash = {
        key_schedule.early_secret(Some((&psk.data, psk.ty)))?;
        key_schedule.write_mut().create_psk_binder(cipher_suite, hash.lease())?
      }
      && local_hash.lease() == offered_psk.binder.lease()
    {
      *handshake_path = HandshakePath::Resumed;
      return Ok(Some(idx));
    }
    idx = idx.wrapping_add(1);
  }
  key_schedule.early_secret(None)?;
  Ok(None)
}

fn seek_signature_algorithm_cert<'client, 'rslt, 'server>(
  client: &'client [SignatureTy],
  server: &'server [SignatureTy],
) -> crate::Result<SignatureTy>
where
  'client: 'rslt,
  'server: 'rslt,
{
  for server_el in server {
    if client.iter().any(|client_el| client_el == server_el) {
      return Ok(*server_el);
    }
  }
  Err(TlsError::ServerHasNoCompatibleAlgorithmTy.into())
}
