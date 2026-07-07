use crate::{
  asn1::Asn1DecodeWrapperAux,
  codec::{Decode as _, Encode as _},
  collections::{ArrayVectorCopy, ArrayVectorU8},
  misc::{Lease, SingleTypeStorage},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    DLFT_MAX_FRAGMENT_LENGTH, HandshakePath, MAX_CERTIFICATES, MAX_KEY_SHARES_LEN,
    MaxFragmentLength, NamedGroup, Psk, TlsBuffer, TlsConfig, TlsError, TlsMode, TlsServerEndPoint,
    TlsStream,
    key_schedule::KeySchedule,
    misc::{fetch_rec_from_stream, server_sig_msg},
    protocol::{
      alert::Alert,
      certificate::Certificate,
      certificate_verify::CertificateVerify,
      client_hello::ClientHello,
      encrypted_extensions::EncryptedExtensions,
      finished::Finished,
      handshake::{Handshake, HandshakeType},
      named_group::NamedGroupAgreement,
      record::Record,
      record_content_type::RecordContentType,
      server_hello::ServerHello,
    },
    read_record_info::ReadRecordInfo,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
    tls_hash::{TlsDigest, TlsHash},
  },
  x509::{CvEndEntity, CvIntermediate, SubjectPublicKeyInfo, validate_signature},
};
use core::{
  hint::cold_path,
  mem,
  ops::{ControlFlow, Range},
};

/// Returned by [`TlsConnector::manage_client_records`].
#[derive(Debug, PartialEq)]
pub enum ManageClientRecordsState {
  /// Finished processing client records
  Terminated(ArrayVectorCopy<u8, { 6 + 74 }>),
}

/// Required by [`TlsConnector::manage_remaining_server_records`].
#[derive(Debug)]
pub struct ManageRemainingServerRecordsInput {
  selected_identity: Option<u16>,
  spki_range: Range<usize>,
  tls_server_end_point: TlsServerEndPoint,
  transcript_digest: TlsDigest,
}

/// Returned by [`TlsConnector::manage_remaining_server_records`].
#[derive(Debug, PartialEq)]
pub enum ManageRemainingServerRecordsState {
  /// Received an alert that requires a connection termination.
  Alert(Alert),
  /// It is necessary to fetch more external data
  NeedsMoreData,
  /// Finished processing server records
  Terminated,
}

/// TLS Connector
///
/// Performs TLS handshakes for clients.
#[derive(Debug)]
pub struct TlsConnector<RNG, S, TC> {
  buffer: TlsBuffer,
  config: TC,
  handshake_path: HandshakePath,
  key_schedule: KeySchedule,
  max_fragment_length: u16,
  named_group: NamedGroup,
  psk: Option<Psk>,
  rng: RNG,
  stream: S,
  transcript_hash: TlsHash,
}

impl<RNG, S, TC, TM> TlsConnector<RNG, S, TC>
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
    let named_group = cfg_ref.inner.named_groups.first().copied();
    Self {
      buffer: TlsBuffer::new(),
      config,
      handshake_path: HandshakePath::Full,
      key_schedule,
      max_fragment_length,
      named_group: named_group.unwrap_or(NamedGroup::default()),
      psk: None,
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

  /// See [`MaxFragmentLength`].
  #[inline]
  pub fn set_fragment_length(&mut self, value: MaxFragmentLength) {
    self.max_fragment_length = value.num();
  }

  /// Changes the internal value. See [`Psk`].
  #[inline]
  #[must_use]
  pub fn set_psk(mut self, value: Option<Psk>) -> TlsConnector<RNG, S, TC> {
    self.psk = value;
    self
  }

  /// Underlying stream
  #[inline]
  pub const fn stream(&self) -> &S {
    &self.stream
  }

  /// Mutable version of [`Self::stream`].
  #[inline]
  pub const fn stream_mut(&mut self) -> &mut S {
    &mut self.stream
  }
}

impl<RNG, S, TC, TM> TlsConnector<RNG, S, TC>
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
  pub async fn connect(mut self) -> crate::Result<TlsConnectOutput<RNG, S, TM>> {
    if TM::TY.is_plain_text() {
      return Ok(TlsConnectOutput {
        handshake_path: self.handshake_path,
        named_group: self.named_group,
        rng: self.rng,
        server_end_point: TlsServerEndPoint::new(),
        tls_stream: TlsStream::new(
          self.buffer,
          self.key_schedule,
          self.max_fragment_length,
          self.stream,
          self.config.lease().mode().clone(),
        ),
      });
    }
    let secrets = self.write_client_hello()?;
    self.stream.write_all(&self.buffer.writer_buffer).await?;
    let first_rri = self.fetch_rec_from_stream::<false>(false).await?;
    let mut mrsri = match self.manage_initial_server_record(&first_rri, secrets)? {
      ControlFlow::Continue(mrsri) => mrsri,
      ControlFlow::Break(alert) => {
        self.write_alert(alert).await?;
        return Err(TlsError::AbortedHandshake(alert).into());
      }
    };
    self.buffer.writer_buffer.clear();
    let mut rri = self.fetch_rec_from_stream::<true>(true).await?;
    *self.buffer.reader_buffer.forbid_clear_mut() = true;
    loop {
      match self.manage_remaining_server_records(&mut mrsri, &rri)? {
        ManageRemainingServerRecordsState::Alert(alert) => {
          self.write_alert(alert).await?;
          return Err(TlsError::AbortedHandshake(alert).into());
        }
        ManageRemainingServerRecordsState::NeedsMoreData => {
          rri = self.fetch_rec_from_stream::<false>(true).await?;
        }
        ManageRemainingServerRecordsState::Terminated => break,
      }
    }
    *self.buffer.reader_buffer.forbid_clear_mut() = false;
    match self.manage_client_records()? {
      ManageClientRecordsState::Terminated(data) => {
        self.stream.write_all(&data).await?;
      }
    }
    Ok(TlsConnectOutput {
      handshake_path: self.handshake_path,
      named_group: self.named_group,
      rng: self.rng,
      server_end_point: mrsri.tls_server_end_point,
      tls_stream: TlsStream::new(
        self.buffer,
        self.key_schedule,
        self.max_fragment_length,
        self.stream,
        self.config.lease().mode().clone(),
      ),
    })
  }

  /// Low level operation that must be called after [`Self::manage_remaining_server_records`].
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_client_records(&mut self) -> crate::Result<ManageClientRecordsState> {
    let (_, ksw) = self.key_schedule.split_mut();
    let verify_data = ksw
      .state_mut()
      .create_finished_verify_data(self.transcript_hash.clone().finalize().lease())?;
    let finished = Finished::record_bytes(&verify_data, ksw.state_mut())?;
    self.key_schedule.master_secret::<true>(&self.transcript_hash.clone().finalize())?;
    let mut terminated = ArrayVectorCopy::new();
    let _ = terminated.extend_from_copyable_slices([&[20, 3, 3, 0, 1, 1][..], &finished])?;
    Ok(ManageClientRecordsState::Terminated(terminated))
  }

  /// Low level operation that must be called after [`Self::write_client_hello`].
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_initial_server_record(
    &mut self,
    rri: &ReadRecordInfo,
    secrets: ArrayVectorU8<NamedGroupAgreement, MAX_KEY_SHARES_LEN>,
  ) -> crate::Result<ControlFlow<Alert, ManageRemainingServerRecordsInput>> {
    let current = self.buffer.reader_buffer.current();
    let plaintext = current.get(..rri.plaintext_len).unwrap_or_default();
    match rri.outer_ty {
      RecordContentType::Alert => {
        let dw = &mut TlsDecodeWrapper::from_bytes(plaintext);
        return Ok(ControlFlow::Break(Alert::decode(dw)?));
      }
      RecordContentType::Handshake => {}
      RecordContentType::ApplicationData | RecordContentType::ChangeCipherSpec => {
        return Err(TlsError::InvalidHandshake.into());
      }
    }
    let dw = &mut TlsDecodeWrapper::from_bytes(plaintext);
    let server_hello = Handshake::<ServerHello<'_>>::decode(dw)?;
    let secret = secrets
      .into_iter()
      .find(|el| el.named_group() == server_hello.data.key_share().group)
      .ok_or(TlsError::SecretMismatch)?;
    self.named_group = secret.named_group();
    if server_hello.data.selected_identity().is_none() {
      self.key_schedule.set_cipher_suite(server_hello.data.cipher_suite());
      self.key_schedule.early_secret(None)?;
    }
    self.transcript_hash = self.key_schedule.cipher_suite().hash_new();
    self.transcript_hash.update(self.buffer.writer_buffer.get(5..).unwrap_or_default());
    self.transcript_hash.update(plaintext);
    let shared_secret = secret.diffie_hellman(server_hello.data.key_share().opaque)?;
    self
      .key_schedule
      .handshake_secret::<true>(shared_secret.as_ref(), &self.transcript_hash.clone().finalize())?;
    Ok(ControlFlow::Continue(ManageRemainingServerRecordsInput {
      selected_identity: server_hello.data.selected_identity(),
      spki_range: 0..0,
      tls_server_end_point: TlsServerEndPoint::new(),
      transcript_digest: TlsDigest::default(),
    }))
  }

  /// Low level operation that must be called after [`Self::manage_initial_server_record`].
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_remaining_server_records(
    &mut self,
    mrsri: &mut ManageRemainingServerRecordsInput,
    rri: &ReadRecordInfo,
  ) -> crate::Result<ManageRemainingServerRecordsState> {
    let tls_rec_payload = self.buffer.reader_buffer.current();
    let mut maybe_handshakes = tls_rec_payload.get(..rri.plaintext_len).unwrap_or_default();
    match rri.inner_ty {
      RecordContentType::Alert => {
        let dw = &mut TlsDecodeWrapper::from_bytes(maybe_handshakes);
        return Ok(ManageRemainingServerRecordsState::Alert(Alert::decode(dw)?));
      }
      RecordContentType::ApplicationData => return Err(TlsError::InvalidHandshake.into()),
      RecordContentType::ChangeCipherSpec => {
        return Ok(ManageRemainingServerRecordsState::NeedsMoreData);
      }
      RecordContentType::Handshake => {}
    }
    while !maybe_handshakes.is_empty() {
      let before_len = maybe_handshakes.len();
      let mut dw = TlsDecodeWrapper::from_bytes(maybe_handshakes);
      let hs = Handshake::<&[u8]>::decode(&mut dw)?;
      let curr_handshake_len = before_len.wrapping_sub(dw.bytes().len());
      let curr_handshake_bytes = maybe_handshakes.get(..curr_handshake_len).unwrap_or_default();
      self.transcript_hash.update(curr_handshake_bytes);
      maybe_handshakes = dw.bytes();
      *dw.bytes_mut() = hs.data;
      match hs.msg_type {
        HandshakeType::EncryptedExtensions => {
          let encrypted_extensions = EncryptedExtensions::decode(&mut dw)?;
          if let Some(el) = encrypted_extensions.max_fragment_length() {
            self.max_fragment_length = el.num();
          }
        }
        HandshakeType::CertificateRequest => {
          return Err(TlsError::UnsupportedMtls.into());
        }
        HandshakeType::Certificate => {
          if mrsri.selected_identity.is_some() {
            cold_path();
            return Err(TlsError::CertRecordInAcceptedPsk.into());
          }
          let certificate_record_begin_idx = self.buffer.reader_buffer.antecedent_end_idx();
          Self::manage_certificate(
            certificate_record_begin_idx,
            self.config.lease(),
            &self.key_schedule,
            mrsri,
            &mut dw,
            &self.transcript_hash,
          )?;
        }
        HandshakeType::CertificateVerify => {
          if mrsri.selected_identity.is_some() {
            cold_path();
            return Err(TlsError::CertRecordInAcceptedPsk.into());
          }
          Self::manage_certificate_verify(self.buffer.reader_buffer.filled(), mrsri, &mut dw)?;
          mrsri.transcript_digest = self.transcript_hash.clone().finalize();
        }
        HandshakeType::Finished => {
          let prev = mem::replace(dw.cipher_suite_mut(), self.key_schedule.cipher_suite());
          let finished = Finished::decode(&mut dw)?;
          *dw.cipher_suite_mut() = prev;
          self
            .key_schedule
            .read_mut()
            .state_mut()
            .verify_finished_record(mrsri.transcript_digest.lease(), finished.verify_data())?;
          return Ok(ManageRemainingServerRecordsState::Terminated);
        }
        HandshakeType::ClientHello
        | HandshakeType::EndOfEarlyData
        | HandshakeType::KeyUpdate
        | HandshakeType::MessageHash
        | HandshakeType::NewSessionTicket
        | HandshakeType::ServerHello => {
          return Err(TlsError::InvalidHandshake.into());
        }
      }
    }
    Ok(ManageRemainingServerRecordsState::NeedsMoreData)
  }

  /// Low level operation responsible for informing the local parameters to the remote server. No other method should
  /// be called before it.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn write_client_hello(
    &mut self,
  ) -> crate::Result<ArrayVectorU8<NamedGroupAgreement, MAX_KEY_SHARES_LEN>> {
    if let Some(Psk { cipher_suite, data, ty }) = &self.psk {
      let mut key_schedule = KeySchedule::from_cipher_suite(*cipher_suite);
      key_schedule.early_secret(Some((data.lease(), *ty)))?;
      self.handshake_path = HandshakePath::Resumed;
      self.key_schedule = key_schedule;
    }
    let mut secrets = ArrayVectorU8::new();
    for key_share in &self.config.lease().inner.key_shares {
      secrets.push(key_share.group.agreement(&mut self.rng)?)?;
    }
    let handshake = Handshake::new(
      HandshakeType::ClientHello,
      ClientHello::new(&mut self.rng, &secrets, self.config.lease()),
    );
    let record = Record::new(RecordContentType::Handshake, &handshake);
    self.buffer.writer_buffer.clear();
    record.encode(&mut TlsEncodeWrapper::from_buffer(&mut self.buffer.writer_buffer))?;
    if let Some(Psk { cipher_suite, .. }) = &self.psk {
      let ksw = self.key_schedule.write_mut();
      let writer_buffer = self.buffer.writer_buffer.as_slice_mut();
      let hash_len = usize::from(cipher_suite.hash_len());
      let binder_total_len = hash_len.wrapping_add(1);
      let transcript_len = writer_buffer.len().wrapping_sub(binder_total_len);
      let handshake_bytes = writer_buffer.get(5..transcript_len).unwrap_or_default();
      let transcript_hash = cipher_suite.hash_digest([handshake_bytes]);
      let computed_binder = ksw.create_psk_binder(*cipher_suite, transcript_hash.lease())?;
      let buffer_len = writer_buffer.len();
      if let Some(elem) = writer_buffer.get_mut(buffer_len.wrapping_sub(hash_len)..) {
        elem.copy_from_slice(computed_binder.lease());
      }
    }
    Ok(secrets)
  }

  #[inline]
  async fn fetch_rec_from_stream<const CHECK_CCS: bool>(
    &mut self,
    decrypt: bool,
  ) -> crate::Result<ReadRecordInfo> {
    Ok(
      fetch_rec_from_stream::<_, CHECK_CCS>(
        decrypt.then(|| self.key_schedule.read_mut().state_mut()),
        self.max_fragment_length,
        &mut self.buffer.reader_buffer,
        &mut self.stream,
      )
      .await?
      .ok_or(TlsError::AbruptDisconnect)?,
    )
  }

  fn manage_certificate(
    certificate_record_begin_idx: usize,
    config: &TlsConfig<TM>,
    key_schedule: &KeySchedule,
    mrsri: &mut ManageRemainingServerRecordsInput,
    remote_dw: &mut TlsDecodeWrapper<'_>,
    transcript_hash: &TlsHash,
  ) -> crate::Result<()> {
    if TM::TY.is_unverified() {
      return Ok(());
    }
    let certificate = Certificate::decode(remote_dw)?;
    if let [end_entity, intermediates @ ..] = certificate.certificate_list().as_slice() {
      mrsri.tls_server_end_point.extend_from_copyable_slice(
        key_schedule.cipher_suite().hash_digest([end_entity.certificate_bytes()]).lease(),
      )?;
      let cv_end_entity = {
        let mut dw = crate::codec::DecodeWrapper::new(
          end_entity.certificate_bytes(),
          Asn1DecodeWrapperAux::default(),
        );
        let cert = crate::x509::Certificate::decode(&mut dw)?;
        let spki_offset = spki_offset();
        mrsri.spki_range = dw.decode_aux.spki_range();
        mrsri.spki_range.start = mrsri.spki_range.start.wrapping_add(spki_offset);
        mrsri.spki_range.start = mrsri.spki_range.start.wrapping_add(certificate_record_begin_idx);
        mrsri.spki_range.end = mrsri.spki_range.end.wrapping_add(spki_offset);
        mrsri.spki_range.end = mrsri.spki_range.end.wrapping_add(certificate_record_begin_idx);
        let sig = dw.decode_aux.tbs_cert(end_entity.certificate_bytes()).unwrap_or_default();
        CvEndEntity::from_certificate(cert, sig)?
      };
      let mut cv_intermediates = ArrayVectorU8::<_, MAX_CERTIFICATES>::new();
      for intermediate in intermediates {
        let cv_intermediate = {
          let mut local_dw = crate::codec::DecodeWrapper::new(
            intermediate.certificate_bytes(),
            Asn1DecodeWrapperAux::default(),
          );
          CvIntermediate::from_certificate(
            crate::x509::Certificate::decode(&mut local_dw)?,
            local_dw.decode_aux.tbs_cert(intermediate.certificate_bytes()).unwrap_or_default(),
          )?
        };
        cv_intermediates.push(cv_intermediate)?;
      }
      mrsri.transcript_digest = transcript_hash.clone().finalize();
      drop(cv_end_entity.validate_chain(
        cv_intermediates.as_slice(),
        config.cv_policy(),
        config.trust_anchors(),
      )?);
    }
    Ok(())
  }

  fn manage_certificate_verify(
    filled: &[u8],
    mrsri: &mut ManageRemainingServerRecordsInput,
    remote_dw: &mut TlsDecodeWrapper<'_>,
  ) -> crate::Result<()> {
    if TM::TY.is_unverified() {
      return Ok(());
    }
    let certificate_verify = CertificateVerify::decode(remote_dw)?;
    let msg = server_sig_msg(mrsri.transcript_digest.lease())?;
    validate_signature(
      &msg,
      certificate_verify.signature(),
      &SubjectPublicKeyInfo::<&[u8]>::decode(&mut crate::codec::DecodeWrapper::new(
        filled.get(mrsri.spki_range.clone()).unwrap_or_default(),
        Asn1DecodeWrapperAux::default(),
      ))?,
    )?;
    Ok(())
  }

  #[inline]
  async fn write_alert(&mut self, alert: Alert) -> crate::Result<()> {
    if !alert.is_close_notify() {
      return Ok(());
    }
    let kss = self.key_schedule.write_mut().state_mut();
    if kss.cipher_key().is_empty() {
      let [level, description] = alert.data_bytes();
      self.stream.write_all(&[21, 3, 3, 0, 2, level, description]).await?;
    } else {
      self.stream.write_all(&Alert::record_bytes(alert.data_bytes(), kss)?).await?;
    }
    Ok(())
  }
}

/// Returned by [`TlsConnector::connect`].
#[derive(Debug)]
pub struct TlsConnectOutput<RNG, S, TM> {
  /// See [`HandshakePath`].
  pub handshake_path: HandshakePath,
  /// See [`NamedGroup`].
  pub named_group: NamedGroup,
  /// Random Number Generator
  pub rng: RNG,
  /// See [`TlsServerEndPoint`]
  pub server_end_point: TlsServerEndPoint,
  /// See [`TlsStream`]
  pub tls_stream: TlsStream<S, TM, true>,
}

const fn spki_offset() -> usize {
  11
}
