use crate::{
  codec::{Decode as _, Encode as _},
  collections::{ArrayVectorCopy, ArrayVectorU8},
  crypto::SignatureTy,
  misc::{Lease, SingleTypeStorage},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    Alpn, CHANGE_CIPHER_SPEC, CipherSuite, DLFT_MAX_FRAGMENT_LENGTH, HandshakePath,
    MaxFragmentLength, NamedGroup, ProtocolVersion, TlsBuffer, TlsConfig, TlsError, TlsMode,
    TlsStream,
    key_schedule::KeySchedule,
    misc::{fetch_rec_from_stream, manage_err, server_sig_msg, write_payloads},
    protocol::{
      certificate::{Certificate, CertificateEntry},
      certificate_verify::CertificateVerify,
      client_hello::ClientHello,
      encrypted_extensions::EncryptedExtensions,
      finished::Finished,
      handshake::{Handshake, HandshakeType},
      key_share_entry::KeyShareEntry,
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
    let named_group = cfg_ref
      .inner
      .supported_groups
      .named_group_list
      .first()
      .copied()
      .unwrap_or(NamedGroup::default());
    Self {
      buffer: TlsBuffer::new(),
      config,
      handshake_path: HandshakePath::Full,
      key_schedule,
      max_fragment_length,
      named_group,
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
  pub async fn accept(mut self) -> crate::Result<TlsAcceptOutput<RNG, S, TM>> {
    if TM::TY.is_plain_text() {
      return Ok(TlsAcceptOutput {
        handshake_path: self.handshake_path,
        named_group: self.named_group,
        rng: self.rng,
        tls_stream: TlsStream::new(
          self.buffer,
          self.key_schedule,
          self.max_fragment_length,
          self.stream,
          self.config.lease().mode().clone(),
        )?,
      });
    }
    _trace!(target: crate::tls::_TARGET_HS, "Start");
    let fut = async {
      let first_rri = self.fetch_rec_from_stream::<false, true>(false).await?;
      _trace!(target: crate::tls::_TARGET_HS, "Read ClientHello: {:?}", &first_rri);
      let indices = self.manage_initial_client_record(&first_rri)?;
      let buffer = self.buffer.reader_buffer.buffer_mut();
      let payloads = match indices.as_slice() {
        [idx0, idx1] => {
          &[buffer.get(*idx0..*idx1).unwrap_or_default(), buffer.get(*idx1..).unwrap_or_default()][..]
        }
        [idx0, idx1, idx2, idx3] => &[
          buffer.get(*idx0..*idx1).unwrap_or_default(),
          buffer.get(*idx1..*idx2).unwrap_or_default(),
          buffer.get(*idx2..*idx3).unwrap_or_default(),
          buffer.get(*idx3..).unwrap_or_default(),
        ],
        _ => &[],
      };
      _trace!(target: crate::tls::_TARGET_HS, "Write Records");
      write_payloads(
        RecordContentType::Handshake,
        self.key_schedule.write_mut(),
        self.max_fragment_length,
        payloads,
        &mut self.stream,
        &mut self.buffer.writer_buffer,
      )
      .await?;
      buffer.truncate(indices.first().copied().unwrap_or_default());
      let mut last_rri = self.fetch_rec_from_stream::<true, false>(true).await?;
      if last_rri.outer_ty == RecordContentType::ChangeCipherSpec {
        last_rri = self.fetch_rec_from_stream::<false, false>(true).await?;
      }
      _trace!(target: crate::tls::_TARGET_HS, "Read Finished: {:?}", &last_rri);
      self.manage_final_client_record(&last_rri)?;
      Ok(())
    };
    let rslt = fut.await;
    let kss = self.key_schedule.write_mut().state_mut();
    manage_err::<_, _, true>(kss, rslt, &mut self.stream).await?;
    _trace!(target: crate::tls::_TARGET_HS, "Successful handshake");
    Ok(TlsAcceptOutput {
      handshake_path: self.handshake_path,
      named_group: self.named_group,
      rng: self.rng,
      tls_stream: TlsStream::new(
        self.buffer,
        self.key_schedule,
        self.max_fragment_length,
        self.stream,
        self.config.lease().mode().clone(),
      )?,
    })
  }

  /// Low level operation responsible for processing data sent by clients. No other method should
  /// be called before it.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_initial_client_record(
    &mut self,
    rri: &ReadRecordInfo,
  ) -> crate::Result<ArrayVectorCopy<usize, 4>> {
    let RecordContentType::Handshake = rri.outer_ty else {
      return Err(TlsError::InvalidHandshake.into());
    };
    let output = self.negotiate(rri)?;
    self.buffer.reader_buffer.clear_if_exhausted();
    let reader_buffer = self.buffer.reader_buffer.buffer_mut();
    let mut curr_idx = reader_buffer.len();
    let mut indices = ArrayVectorCopy::new();
    let encrypted_extensions = Handshake::new(
      HandshakeType::EncryptedExtensions,
      EncryptedExtensions::new(output.alpn, output.max_fragment_length, None, None),
    );
    encrypted_extensions.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
    self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());
    drop(indices.push(curr_idx));
    curr_idx = reader_buffer.len();
    drop(indices.push(curr_idx));
    let mut cert_list = ArrayVectorU8::new();
    for (_sig, bytes) in &self.config.lease().inner.public_key {
      cert_list.push(CertificateEntry::new(bytes))?;
    }
    {
      let ty = HandshakeType::Certificate;
      let certificate = Handshake::new(ty, Certificate::new(cert_list, &[]));
      certificate.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
      self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());
      curr_idx = reader_buffer.len();
      let _rslt = indices.push(curr_idx);
    }
    let signature = self.config.lease().inner.secret_key.peek(
      &mut (&mut self.buffer.writer_buffer).into(),
      |secret_key| {
        let mut sign_key = output.signature_ty.sign_key_from_pkcs8(*secret_key)?;
        let msg = server_sig_msg(self.transcript_hash.clone().finalize().lease())?;
        sign_key.sign(&mut self.rng, &msg)
      },
    )??;
    {
      let certificate_verify = Handshake::new(
        HandshakeType::CertificateVerify,
        CertificateVerify::new(output.signature_ty, signature.as_ref()),
      );
      certificate_verify.encode(&mut TlsEncodeWrapper::from_buffer(reader_buffer))?;
      self.transcript_hash.update(reader_buffer.get(curr_idx..).unwrap_or_default());
      curr_idx = reader_buffer.len();
      let _rslt = indices.push(curr_idx);
    }
    let verify_data = self
      .key_schedule
      .write_mut()
      .state_mut()
      .create_finished_verify_data(self.transcript_hash.clone().finalize().lease())?;
    let finished = Handshake::new(HandshakeType::Finished, Finished::new(verify_data.as_slice()));
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
    if rri.outer_ty != RecordContentType::ApplicationData
      || rri.inner_ty != RecordContentType::Handshake
    {
      return Err(TlsError::InvalidHandshake.into());
    }
    let current = self.buffer.reader_buffer.current();
    let plaintext = current.get(..rri.plaintext_len).unwrap_or_default();
    let mut dw = TlsDecodeWrapper::from_bytes(plaintext);
    let hs = Handshake::<&[u8]>::decode(&mut dw)?;
    if hs.msg_type != HandshakeType::Finished {
      return Err(TlsError::InvalidHandshake.into());
    }
    *dw.bytes_mut() = hs.data;
    *dw.cipher_suite_mut() = self.key_schedule.cipher_suite();
    let finished = Finished::decode(&mut dw)?;
    if self
      .key_schedule
      .read_mut()
      .state_mut()
      .verify_finished_record(
        self.transcript_hash.clone().finalize().lease(),
        finished.verify_data(),
      )
      .is_err()
    {
      return Err(TlsError::DigestCheckFailed.into());
    }
    self.key_schedule.master_secret::<false>(&self.transcript_hash.clone().finalize())?;
    Ok(())
  }

  #[inline]
  async fn fetch_rec_from_stream<const CHECK_CCS: bool, const IS_CH: bool>(
    &mut self,
    decrypt: bool,
  ) -> crate::Result<ReadRecordInfo> {
    Ok(
      fetch_rec_from_stream::<_, CHECK_CCS, IS_CH>(
        decrypt.then(|| self.key_schedule.read_mut().state_mut()),
        self.max_fragment_length,
        &mut self.buffer.reader_buffer,
        &mut self.stream,
      )
      .await?
      .ok_or(TlsError::AbruptDisconnect)?,
    )
  }

  #[inline]
  fn negotiate(&mut self, rri: &ReadRecordInfo) -> crate::Result<NegotiateOutput>
  where
    TM: TlsMode,
  {
    let current = self.buffer.reader_buffer.current();
    let client_hello_bytes = current.get(..rri.plaintext_len).unwrap_or_default();
    let client_hello = Handshake::<ClientHello<_, TlsConfigInner<_, TM>>>::decode(
      &mut TlsDecodeWrapper::from_bytes(client_hello_bytes),
    )?;
    if !client_hello
      .data
      .supported_versions()
      .versions
      .iter()
      .copied()
      .any(|el| el == ProtocolVersion::Tls13)
    {
      return Err(
        TlsError::UnsupportedTlsVersion(
          client_hello.data.supported_versions().versions.last().copied(),
        )
        .into(),
      );
    }
    let cipher_suite = seek_cipher_suite(
      &client_hello.data.tls_config().cipher_suites,
      &self.config.lease().inner.cipher_suites,
    )?;
    self.key_schedule.set_cipher_suite(cipher_suite);
    self.key_schedule.early_secret()?;
    self.transcript_hash = cipher_suite.hash_new();
    self.transcript_hash.update(client_hello_bytes);
    let key_share = seek_key_share(
      &client_hello.data.generic().client_shares,
      &self.config.lease().inner.supported_groups.named_group_list,
    )?;
    let alpn = seek_alpn(&client_hello.data.tls_config().alpn, &self.config.lease().inner.alpn);
    self.named_group = key_share.group;
    let max_fragment_length = client_hello.data.tls_config().max_fragment_length;
    if let Some(elem) = max_fragment_length {
      self.max_fragment_length = elem.num();
    }
    let leaf_sig_ty =
      self.config.lease().inner.public_key.first().ok_or(TlsError::EmptySetOfCertificates)?.0;
    let signature_ty = seek_signature_ty(
      &client_hello.data.tls_config().signature_algorithms.signature_schemes,
      &[leaf_sig_ty],
    )?;
    if let Some(elem) = &client_hello.data.tls_config().signature_algorithms_cert {
      let _ = seek_signature_ty_cert(&elem.supported_groups, &[leaf_sig_ty])?;
    }
    let legacy_session_id = *client_hello.data.legacy_session_id();
    let agreement = key_share.group.agreement(&mut self.rng)?;
    let ephemeral_pk = agreement.public_key()?;
    let secret = agreement.diffie_hellman(key_share.opaque)?;
    let writer_buffer = &mut self.buffer.writer_buffer;
    let server_hello_rec = Record::new(
      RecordContentType::Handshake,
      ProtocolVersion::Tls12,
      Handshake::new(
        HandshakeType::ServerHello,
        ServerHello::new(
          cipher_suite,
          false,
          KeyShareEntry::new(key_share.group, ephemeral_pk.as_ref()),
          legacy_session_id,
          &mut self.rng,
        ),
      ),
    );
    writer_buffer.clear();
    server_hello_rec.encode(&mut TlsEncodeWrapper::from_buffer(writer_buffer))?;
    self.transcript_hash.update(writer_buffer.get(5..).unwrap_or_default());
    writer_buffer.extend_from_copyable_slice(&CHANGE_CIPHER_SPEC)?;
    self
      .key_schedule
      .handshake_secret::<false>(secret.as_ref(), &self.transcript_hash.clone().finalize())?;
    Ok(NegotiateOutput { alpn, max_fragment_length, signature_ty })
  }
}

/// Returned by [`TlsAcceptor::accept`].
#[derive(Debug)]
pub struct TlsAcceptOutput<RNG, S, TM> {
  /// See [`HandshakePath`].
  pub handshake_path: HandshakePath,
  /// See [`NamedGroup`].
  pub named_group: NamedGroup,
  /// Random Number Generator
  pub rng: RNG,
  /// See [`TlsStream`]
  pub tls_stream: TlsStream<S, TM, false>,
}

#[derive(Debug)]
struct NegotiateOutput {
  alpn: Option<Alpn>,
  max_fragment_length: Option<MaxFragmentLength>,
  signature_ty: SignatureTy,
}

fn seek_alpn(client_opt: &Option<Alpn>, server_opt: &Option<Alpn>) -> Option<Alpn> {
  let Some(client) = client_opt else {
    // Client did not request this extension
    return None;
  };
  let Some(server) = server_opt else {
    // Server did not want to support this extension
    return None;
  };
  let mut common = Alpn::default();
  for server_el in &server.protocol_name_list {
    for client_el in &client.protocol_name_list {
      if server_el == client_el {
        let _rslt = common.protocol_name_list.push(*server_el);
      }
    }
  }
  Some(common)
}

fn seek_cipher_suite(client: &[CipherSuite], server: &[CipherSuite]) -> crate::Result<CipherSuite> {
  for elem in server {
    if client.contains(elem) {
      return Ok(*elem);
    }
  }
  Err(TlsError::ServerHasNoCompatibleCypherSuite.into())
}

fn seek_key_share<'client, 'rslt, 'server>(
  client: &'client [KeyShareEntry<&'client [u8]>],
  server: &'server [NamedGroup],
) -> crate::Result<KeyShareEntry<&'rslt [u8]>>
where
  'client: 'rslt,
  'server: 'rslt,
{
  for server_el in server {
    let Some(client_el) = client.iter().find(|client_el| client_el.group == *server_el) else {
      continue;
    };
    return Ok(*client_el);
  }
  Err(TlsError::ServerHasNoCompatibleKeyShare.into())
}

fn seek_signature_ty<'client, 'rslt, 'server>(
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

fn seek_signature_ty_cert<'client, 'rslt, 'server>(
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
  Err(TlsError::ServerHasNoCompatibleAlgorithmTyForCert.into())
}
