use crate::{
  asn1::Asn1DecodeWrapper,
  codec::{Decode as _, Encode},
  collection::{ArrayVectorU8, Vector},
  crypto::CryptoError,
  misc::{PartitionedFilledBuffer, SuffixWriter},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    CipherSuite, MAX_CERTIFICATES, MAX_KEY_SHARES_LEN, Psk, SERVER_SIG_CTX, TlsConfig, TlsError,
    TlsMode, TlsModeVerified, TlsStream,
    decode_wrapper::DecodeWrapper,
    encode_wrapper::EncodeWrapper,
    key_schedule::{KeySchedule, KeyScheduleServer},
    misc::fetch_rec_from_stream,
    protocol::{
      alert::Alert,
      certificate::Certificate,
      certificate_request::CertificateRequest,
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
  },
  x509::{CvEndEntity, CvIntermediate},
};

/// Basically performs the TLS handshake
#[derive(Debug)]
pub struct TlsConnector<S, TM> {
  has_psk: bool,
  key_schedule: KeySchedule,
  stream: S,
  tm: TM,
}

impl<S> TlsConnector<S, TlsModeVerified> {
  /// All parameters are provided by the user.
  #[inline]
  pub fn from_stream(stream: S) -> Self {
    Self {
      has_psk: false,
      key_schedule: KeySchedule::from_cipher_suite_ty(CipherSuite::Aes128GcmSha256),
      stream,
      tm: TlsModeVerified,
    }
  }
}

impl<S, TM> TlsConnector<S, TM>
where
  S: Stream,
  TM: TlsMode,
{
  /// High level operation that automatically performs a asynchronous full handshake.
  ///
  /// Low level operations must not be mixed with high level operations.
  pub async fn connect<RNG>(
    mut self,
    network_buffer: &mut PartitionedFilledBuffer,
    psk: Option<Psk<'_>>,
    rng: &mut RNG,
    tls_config: &TlsConfig<'_>,
    write_buffer: &mut Vector<u8>,
  ) -> crate::Result<TlsStream<S, TM, true>>
  where
    RNG: CryptoRng,
  {
    if TM::TY.is_plain_text() {
      return Ok(TlsStream::new(self.stream, self.tm));
    }
    let secrets = self.write_client_hello(psk, rng, tls_config, write_buffer)?;
    self.stream.write_all(write_buffer).await?;
    let ty = fetch_rec_from_stream(network_buffer, &mut self.stream).await?.1;
    if !self.manage_initial_server_record(network_buffer, secrets, ty, write_buffer)? {
      self.stream.write_all(write_buffer).await?;
      return Err(TlsError::AbortedHandshake.into());
    }
    loop {
      let (header, ty) = fetch_rec_from_stream(network_buffer, &mut self.stream).await?;
      match self.manage_remaining_server_records(
        header,
        network_buffer,
        tls_config,
        ty,
        write_buffer,
      )? {
        Some(false) => {}
        Some(true) => break,
        None => panic!(),
      }
    }
    Ok(TlsStream::new(self.stream, self.tm))
  }

  /// Low level operation that must be called after [`Self::write_client_hello`].
  ///
  /// Returns `false` if the connection was aborted by the server.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_initial_server_record(
    &mut self,
    network_buffer: &mut PartitionedFilledBuffer,
    secrets: ArrayVectorU8<NamedGroupAgreement, MAX_KEY_SHARES_LEN>,
    ty: RecordContentType,
    write_buffer: &mut Vector<u8>,
  ) -> crate::Result<bool> {
    match ty {
      RecordContentType::Handshake => {}
      RecordContentType::Alert => {
        self.read_and_write_alert(network_buffer, write_buffer)?;
        return Ok(false);
      }
      _ => return Err(TlsError::InvalidHandshake.into()),
    }
    let server_hello =
      Handshake::<ServerHello>::decode(&mut DecodeWrapper::from_bytes(network_buffer.current()))?;
    let mut secret_opt = None;
    for secret in secrets {
      if secret.named_group() == server_hello.data.key_share().group {
        secret_opt = Some(secret);
        break;
      }
    }
    let secret = secret_opt.ok_or(TlsError::SecretMismatch)?;
    if !self.has_psk {
      self.key_schedule.set_cipher_suite_ty(server_hello.data.cipher_suite());
      self.key_schedule.early_secret(None)?;
    }
    let shared_secret = secret.diffie_hellman(server_hello.data.key_share().opaque)?;
    self.key_schedule.handshake_secret(shared_secret.as_ref())?;
    Ok(true)
  }

  /// Low level operation that must be called after [`Self::write_client_hello`].
  ///
  /// Returns `None` if the connection was aborted by the server or `Some(false)` if
  /// this method needs to be called again.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn manage_remaining_server_records(
    &mut self,
    header: [u8; 5],
    network_buffer: &mut PartitionedFilledBuffer,
    tls_config: &TlsConfig<'_>,
    ty: RecordContentType,
    write_buffer: &mut Vector<u8>,
  ) -> crate::Result<Option<bool>> {
    match ty {
      RecordContentType::Alert => {
        self.read_and_write_alert(network_buffer, write_buffer)?;
        return Ok(None);
      }
      RecordContentType::ApplicationData => {}
      _ => return Err(TlsError::InvalidHandshake.into()),
    }
    let payload = decrypt_record(
      self.key_schedule.cipher_suite(),
      header,
      self.key_schedule.server_mut(),
      network_buffer,
    )?;
    let mut remote_dw = DecodeWrapper::from_bytes(payload);
    let hs = Handshake::<&[u8]>::decode(&mut remote_dw)?;
    *remote_dw.bytes_mut() = hs.data;
    let mut certificate_request = None;
    match hs.msg_type {
      HandshakeType::EncryptedExtensions => {
        let _encrypted_extensions = EncryptedExtensions::decode(&mut remote_dw)?;
      }
      HandshakeType::CertificateRequest => {
        certificate_request = Some(CertificateRequest::decode(&mut remote_dw)?);
      }
      HandshakeType::Certificate => {
        let certificate = Certificate::decode(&mut remote_dw)?;
        if let [first, intermediates @ ..] = certificate.certificate_list().as_slice() {
          let cv_cert = {
            let mut local_dw = crate::codec::DecodeWrapper::new(
              first.certificate_bytes(),
              Asn1DecodeWrapper::default(),
            );
            let local_cert = crate::x509::Certificate::decode(&mut local_dw)?;
            CvEndEntity::try_from(local_cert)?
          };
          let mut cv_intermediates = ArrayVectorU8::<_, MAX_CERTIFICATES>::new();
          for intermediate in intermediates {
            let cv_intermediate = {
              let mut local_dw = crate::codec::DecodeWrapper::new(
                intermediate.certificate_bytes(),
                Asn1DecodeWrapper::default(),
              );
              let local_cert = crate::x509::Certificate::decode(&mut local_dw)?;
              CvIntermediate::try_from(local_cert)?
            };
            cv_intermediates.push(cv_intermediate)?;
          }
          cv_cert.validate_chain(
            cv_intermediates.as_slice(),
            tls_config.cv_policy(),
            tls_config.trust_anchors(),
          )?;
        }
      }
      HandshakeType::CertificateVerify => {
        let certificate_verify = CertificateVerify::decode(&mut remote_dw)?;
        let cv_cert: crate::x509::CvCertificate<'_, '_, true> = {
          let mut local_dw = crate::codec::DecodeWrapper::new(
            certificate_verify.signature(),
            Asn1DecodeWrapper::default(),
          );
          let local_cert = crate::x509::Certificate::decode(&mut local_dw)?;
          CvEndEntity::try_from(local_cert)?
        };
        let mut msg = [0; 64];
        msg[..SERVER_SIG_CTX.len()].copy_from_slice(SERVER_SIG_CTX.as_bytes());
        cv_cert.validate_signature(&msg, certificate_verify.signature())?;
      }
      HandshakeType::Finished => {
        let finished = Finished::decode(&mut remote_dw)?;
        self.key_schedule.verify_finished(&[], finished.verify_data())?;
        return Ok(Some(true));
      }
      _ => {
        return Err(TlsError::InvalidHandshake.into());
      }
    }
    Ok(Some(false))
  }

  pub fn tls_mode<_TM>(self, tm: _TM) -> TlsConnector<S, _TM> {
    TlsConnector { has_psk: self.has_psk, key_schedule: self.key_schedule, stream: self.stream, tm }
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
    write_buffer: &mut Vector<u8>,
  ) -> crate::Result<ArrayVectorU8<NamedGroupAgreement, MAX_KEY_SHARES_LEN>>
  where
    RNG: CryptoRng,
  {
    if let Some(Psk { cipher_suite_ty, .. }) = psk {
      let mut key_schedule = KeySchedule::from_cipher_suite_ty(cipher_suite_ty);
      key_schedule.early_secret(psk.map(|Psk { data, psk_ty, .. }| (data, psk_ty)))?;
      self.key_schedule = key_schedule;
      self.has_psk = true;
    }
    let mut secrets = ArrayVectorU8::new();
    for key_share in &tls_config.key_shares {
      secrets.push(key_share.group.agreement(rng)?)?;
    }
    let handshake = Handshake {
      data: ClientHello::new(rng, &secrets, &tls_config)?,
      msg_type: HandshakeType::ClientHello,
    };
    let record = Record::new(RecordContentType::Handshake, &handshake);
    write_buffer.clear();
    let mut ew = EncodeWrapper::from_buffer(SuffixWriter::new(0, write_buffer));
    record.encode(&mut ew)?;
    Ok(secrets)
  }

  /// Low level operation that must be called after [`Self::manage_remaining_server_records`] is concluded.
  ///
  /// High level operations must not be mixed with low level operations.
  #[inline]
  pub fn write_final_records<RNG>() {}

  fn read_and_write_alert(
    &mut self,
    network_buffer: &mut PartitionedFilledBuffer,
    write_buffer: &mut Vector<u8>,
  ) -> crate::Result<()> {
    let alert = Alert::decode(&mut DecodeWrapper::from_bytes(network_buffer.current()))?;
    write_buffer.clear();
    let mut ew = EncodeWrapper::from_buffer(SuffixWriter::new(0, write_buffer));
    Record::new(RecordContentType::Alert, alert).encode(&mut ew)?;
    Ok(())
  }
}

pub(crate) fn decrypt_record<'record>(
  cipher_suite: CipherSuite,
  header: [u8; 5],
  key_schedule: &mut KeyScheduleServer,
  network_buffer: &'record mut PartitionedFilledBuffer,
) -> crate::Result<&'record [u8]> {
  let nonce = key_schedule.state().nonce();
  let cipher_key = key_schedule.state().cipher_key();
  let record = network_buffer.current_mut();
  cipher_suite.aes_decrypt(&header, record, nonce, &cipher_key)?;
  let [plaintext @ .., b0, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] = record else {
    return Err(CryptoError::InvalidAesData.into());
  };
  let mut payload = plaintext;
  if *b0 == 0 {
    if let Some((idx, _)) = payload.iter().enumerate().rfind(|(_, b)| **b != 0) {
      payload = payload.get_mut(..idx).unwrap_or_default();
    }
    let [rest @ .., last] = payload else {
      panic!();
    };
    payload = rest;
    RecordContentType::try_from(*last)?
  } else {
    RecordContentType::try_from(*b0)?
  };
  key_schedule.state_mut().increment_counter();
  Ok(payload)
}
