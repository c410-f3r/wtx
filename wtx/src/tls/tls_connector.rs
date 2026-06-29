use crate::{
  asn1::Asn1DecodeWrapperAux,
  codec::{Decode as _, Encode as _},
  collections::ArrayVectorU8,
  misc::{Either, Lease, SingleTypeStorage},
  rng::CryptoRng,
  stream::{Stream, StreamReadItem},
  tls::{
    HandshakePath, MAX_CERTIFICATES, MAX_KEY_SHARES_LEN, NamedGroup, Psk, SERVER_SIG_CTX,
    TlsBuffer, TlsConfig, TlsError, TlsMode, TlsServerEndPoint, TlsStream,
    key_schedule::KeySchedule,
    misc::fetch_rec_from_stream,
    protocol::{
      alert::Alert,
      certificate::Certificate,
      certificate_verify::CertificateVerify,
      change_cipher_spec::ChangeCipherSpec,
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
  },
  x509::{CvEndEntity, CvIntermediate},
};
use core::{mem, ops::ControlFlow};

/// Returned by [`TlsConnector::manage_client_records`].
#[derive(Debug, PartialEq)]
pub enum ManageClientRecordsState {
  /// Finished processing client records
  Terminated(ArrayVectorU8<u8, 70>),
}

/// Returned by [`TlsConnector::manage_remaining_server_records`].
#[derive(Debug, PartialEq)]
pub enum ManageRemainingServerRecordsState {
  /// Received an alert that requires a connection termination.
  Alert([u8; 2]),
  /// It is necessary to fetch more external data
  NeedsMoreData,
  /// Finished processing server records
  Terminated(TlsServerEndPoint),
}

/// TLS Connector
///
/// Performs TLS handshakes for clients.
#[derive(Debug)]
pub struct TlsConnector<RNG, S, TC> {
  buffer: TlsBuffer,
  config: TC,
  handshake_path: HandshakePath,
  has_psk: bool,
  key_schedule: KeySchedule,
  named_group: NamedGroup,
  psk: Option<Psk>,
  rng: RNG,
  stream: S,
}

impl<RNG, S, TC, TM> TlsConnector<RNG, S, TC>
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
      has_psk: false,
      key_schedule: KeySchedule::default(),
      named_group,
      psk: None,
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
  pub fn set_rng<_RNG>(self, value: _RNG) -> TlsConnector<_RNG, S, TC> {
    TlsConnector {
      buffer: self.buffer,
      config: self.config,
      handshake_path: HandshakePath::Full,
      has_psk: self.has_psk,
      key_schedule: self.key_schedule,
      named_group: self.named_group,
      psk: self.psk,
      rng: value,
      stream: self.stream,
    }
  }

  /// Changes the internal value. See [`Psk`].
  #[inline]
  #[must_use]
  pub fn set_psk(self, value: Option<Psk>) -> TlsConnector<RNG, S, TC> {
    TlsConnector {
      buffer: self.buffer,
      config: self.config,
      handshake_path: HandshakePath::Full,
      has_psk: self.has_psk,
      key_schedule: self.key_schedule,
      named_group: self.named_group,
      psk: value,
      rng: self.rng,
      stream: self.stream,
    }
  }

  /// Underlying stream
  #[inline]
  pub const fn stream(&mut self) -> &S {
    &mut self.stream
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
  pub async fn connect(mut self) -> crate::Result<StreamReadItem<TlsConnectRslt<RNG, S, TM>>> {
    if TM::TY.is_plain_text() {
      return Ok(StreamReadItem::from_item(TlsConnectRslt {
        handshake_path: self.handshake_path,
        named_group: self.named_group,
        rng: self.rng,
        server_end_point: TlsServerEndPoint::new(),
        stream: TlsStream::new(
          self.buffer,
          self.key_schedule,
          self.stream,
          self.config.lease().mode().clone(),
        ),
      }));
    }

    let secrets = self.write_client_hello()?;
    self.stream.write_all(&self.buffer.writer_buffer).await?;
    let Some(first_rri) = self.fetch_rec_from_stream(false).await?.opt() else {
      return Ok(StreamReadItem::empty_cold());
    };
    if let ControlFlow::Break(alert) = self.manage_initial_server_record(&first_rri, secrets)? {
      self.write_alert(alert).await?;
      return Err(TlsError::AbortedHandshake.into());
    }
    let tls_server_end_point = loop {
      let Some(rri) = self.fetch_rec_from_stream(true).await?.opt() else {
        return Ok(StreamReadItem::empty_cold());
      };
      match self.manage_remaining_server_records(&rri)? {
        ManageRemainingServerRecordsState::Alert(alert) => {
          self.write_alert(alert).await?;
          return Err(TlsError::AbortedHandshake.into());
        }
        ManageRemainingServerRecordsState::NeedsMoreData => {}
        ManageRemainingServerRecordsState::Terminated(el) => break el,
      }
    };
    match self.manage_client_records()? {
      ManageClientRecordsState::Terminated(data) => {
        self.stream.write_all(&data).await?;
      }
    }
    Ok(StreamReadItem::from_item(TlsConnectRslt {
      handshake_path: self.handshake_path,
      named_group: self.named_group,
      rng: self.rng,
      server_end_point: tls_server_end_point,
      stream: TlsStream::new(
        self.buffer,
        self.key_schedule,
        self.stream,
        self.config.lease().mode().clone(),
      ),
    }))
  }

  /// Low level operation that must be called after [`Self::manage_remaining_server_records`].
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_client_records(&mut self) -> crate::Result<ManageClientRecordsState> {
    let kss = self.key_schedule.read_mut();
    let verify_data = kss.state_mut().create_finished_verify_data(&[])?;
    let finished = Finished::record_bytes(&verify_data, kss.state_mut())?;
    self.key_schedule.master_secret()?;
    Ok(ManageClientRecordsState::Terminated(finished))
  }

  /// Low level operation that must be called after [`Self::write_client_hello`].
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_initial_server_record(
    &mut self,
    rri: &ReadRecordInfo,
    secrets: ArrayVectorU8<NamedGroupAgreement, MAX_KEY_SHARES_LEN>,
  ) -> crate::Result<ControlFlow<[u8; 2], ()>> {
    match rri.outer_ty {
      RecordContentType::Alert => {
        let dw = &mut TlsDecodeWrapper::from_bytes(self.buffer.reader_buffer.current());
        return Ok(ControlFlow::Break(Alert::decode(dw)?.data_bytes()));
      }
      RecordContentType::Handshake => {}
      RecordContentType::ApplicationData | RecordContentType::ChangeCipherSpec => {
        return Err(TlsError::InvalidHandshake.into());
      }
    }
    let server_hello = Handshake::<ServerHello<'_>>::decode(&mut TlsDecodeWrapper::from_bytes(
      self.buffer.reader_buffer.current(),
    ))?;
    let mut secret_opt = None;
    for secret in secrets {
      if secret.named_group() == server_hello.data.key_share().group {
        secret_opt = Some(secret);
        break;
      }
    }
    let secret = secret_opt.ok_or(TlsError::SecretMismatch)?;
    self.named_group = secret.named_group();
    if !self.has_psk {
      self.key_schedule.set_cipher_suite(server_hello.data.cipher_suite());
      self.key_schedule.early_secret(None)?;
    }
    let shared_secret = secret.diffie_hellman(server_hello.data.key_share().opaque)?;
    self.key_schedule.handshake_secret(shared_secret.as_ref())?;
    Ok(ControlFlow::Continue(()))
  }

  /// Low level operation that must be called after [`Self::manage_initial_server_record`].
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_remaining_server_records(
    &mut self,
    rri: &ReadRecordInfo,
  ) -> crate::Result<ManageRemainingServerRecordsState> {
    match rri.outer_ty {
      RecordContentType::Alert => {
        let dw = &mut TlsDecodeWrapper::from_bytes(self.buffer.reader_buffer.current());
        return Ok(ManageRemainingServerRecordsState::Alert(Alert::decode(dw)?.data_bytes()));
      }
      RecordContentType::ApplicationData => {}
      RecordContentType::Handshake => {
        let dw = &mut TlsDecodeWrapper::from_bytes(self.buffer.reader_buffer.current());
        let _ = Handshake::<ChangeCipherSpec>::decode(dw)?;
        return Ok(ManageRemainingServerRecordsState::NeedsMoreData);
      }
      RecordContentType::ChangeCipherSpec => return Err(TlsError::InvalidHandshake.into()),
    }
    let mut remote_dw = TlsDecodeWrapper::from_bytes(
      self.buffer.reader_buffer.current().get(..rri.plaintext_len).unwrap_or_default(),
    );
    let hs = Handshake::<&[u8]>::decode(&mut remote_dw)?;
    *remote_dw.bytes_mut() = hs.data;
    let mut tls_server_end_point = TlsServerEndPoint::new();
    match hs.msg_type {
      HandshakeType::EncryptedExtensions => {
        let _encrypted_extensions = EncryptedExtensions::decode(&mut remote_dw)?;
      }
      HandshakeType::CertificateRequest => {
        return Err(TlsError::UnsupportedMtls.into());
      }
      HandshakeType::Certificate => {
        Self::manage_certificate(
          self.config.lease(),
          &self.key_schedule,
          &mut remote_dw,
          &mut tls_server_end_point,
        )?;
      }
      HandshakeType::CertificateVerify => {
        let certificate_verify = CertificateVerify::decode(&mut remote_dw)?;
        let cv_cert: crate::x509::CvCertificate<&[u8], true> = {
          let mut local_dw = crate::codec::DecodeWrapper::new(
            certificate_verify.signature(),
            Asn1DecodeWrapperAux::default(),
          );
          let signature = local_dw.decode_aux.tbs_cert(local_dw.bytes).unwrap_or_default();
          CvEndEntity::from_certificate(
            crate::x509::Certificate::decode(&mut local_dw)?,
            signature,
          )?
        };
        let mut msg = [0; 64];
        if let Some(elem) = msg.get_mut(..SERVER_SIG_CTX.len()) {
          elem.copy_from_slice(SERVER_SIG_CTX.as_bytes());
        }
        cv_cert.validate_signature(&msg, certificate_verify.signature())?;
      }
      HandshakeType::Finished => {
        let prev = mem::replace(remote_dw.cipher_suite_mut(), self.key_schedule.cipher_suite());
        let finished = Finished::decode(&mut remote_dw)?;
        *remote_dw.cipher_suite_mut() = prev;
        self
          .key_schedule
          .read_mut()
          .state_mut()
          .verify_finished_record(&[], finished.verify_data())?;
        return Ok(ManageRemainingServerRecordsState::Terminated(tls_server_end_point));
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
    if let Some(Psk { cipher_suite, data, psk_ty }) = &self.psk {
      let mut key_schedule = KeySchedule::from_cipher_suite(*cipher_suite);
      key_schedule.early_secret(Some((data.lease(), *psk_ty)))?;
      self.key_schedule = key_schedule;
      self.has_psk = true;
    }
    let mut secrets = ArrayVectorU8::new();
    for key_share in &self.config.lease().inner.key_shares {
      secrets.push(key_share.group.agreement(&mut self.rng)?)?;
    }
    let handshake = Handshake {
      data: ClientHello::new(&mut self.rng, &secrets, self.config.lease()),
      msg_type: HandshakeType::ClientHello,
    };
    let record = Record::new(RecordContentType::Handshake, &handshake);
    self.buffer.writer_buffer.clear();
    record.encode(&mut TlsEncodeWrapper::from_buffer(self.buffer.writer_buffer.suffix_pusher()))?;
    if let Some(Psk { cipher_suite, .. }) = &self.psk {
      let writer_buffer = self.buffer.writer_buffer.as_slice_mut();
      let hash_len = usize::from(cipher_suite.hash_len());
      let binder_total_len = hash_len.wrapping_add(1);
      let transcript_len = writer_buffer.len().wrapping_sub(binder_total_len);
      let handshake_bytes = writer_buffer.get(5..transcript_len).unwrap_or_default();
      let transcript_hash = cipher_suite.hash_digest([handshake_bytes]);
      let computed_binder =
        self.key_schedule.write_mut().create_psk_binder(*cipher_suite, transcript_hash.lease())?;
      let buffer_len = writer_buffer.len();
      if let Some(target_slice) = writer_buffer.get_mut(buffer_len.wrapping_sub(hash_len)..) {
        target_slice.copy_from_slice(computed_binder.lease());
      }
    }
    Ok(secrets)
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

  fn manage_certificate(
    config: &TlsConfig<TM>,
    key_schedule: &KeySchedule,
    remote_dw: &mut TlsDecodeWrapper<'_>,
    tls_server_end_point: &mut ArrayVectorU8<u8, 48>,
  ) -> Result<(), crate::Error> {
    let certificate = Certificate::decode(remote_dw)?;
    if let [first, intermediates @ ..] = certificate.certificate_list().as_slice() {
      match key_schedule.cipher_suite().hash_digest([first.certificate_bytes()]) {
        Either::Left(el) => tls_server_end_point.extend_from_copyable_slice(&el)?,
        Either::Right(el) => tls_server_end_point.extend_from_copyable_slice(&el)?,
      }
      let cv_cert = {
        let mut local_dw = crate::codec::DecodeWrapper::new(
          first.certificate_bytes(),
          Asn1DecodeWrapperAux::default(),
        );
        let signature = local_dw.decode_aux.tbs_cert(local_dw.bytes).unwrap_or_default();
        CvEndEntity::from_certificate(crate::x509::Certificate::decode(&mut local_dw)?, signature)?
      };
      let mut cv_intermediates = ArrayVectorU8::<_, MAX_CERTIFICATES>::new();
      for intermediate in intermediates {
        let cv_intermediate = {
          let mut local_dw = crate::codec::DecodeWrapper::new(
            intermediate.certificate_bytes(),
            Asn1DecodeWrapperAux::default(),
          );
          let signature = local_dw.decode_aux.tbs_cert(local_dw.bytes).unwrap_or_default();
          CvIntermediate::from_certificate(
            crate::x509::Certificate::decode(&mut local_dw)?,
            signature,
          )?
        };
        cv_intermediates.push(cv_intermediate)?;
      }
      drop(cv_cert.validate_chain(
        cv_intermediates.as_slice(),
        config.cv_policy(),
        config.trust_anchors(),
      )?);
    }
    Ok(())
  }

  #[inline]
  async fn write_alert(&mut self, alert: [u8; 2]) -> Result<(), crate::Error> {
    let kss = self.key_schedule.write_mut().state_mut();
    self.stream.write_all(&Alert::record_bytes(alert, kss)?).await?;
    Ok(())
  }
}

/// Returned by [`TlsConnector::connect`].
#[derive(Debug)]
pub struct TlsConnectRslt<RNG, S, TM> {
  /// See [`HandshakePath`].
  pub handshake_path: HandshakePath,
  /// See [`NamedGroup`].
  pub named_group: NamedGroup,
  /// Random Number Generator
  pub rng: RNG,
  /// See [`TlsServerEndPoint`]
  pub server_end_point: TlsServerEndPoint,
  /// See [`TlsStream`]
  pub stream: TlsStream<S, TM, true>,
}
