use crate::{
  codec::{Decode, Encode},
  crypto::AEAD_TAG_LEN,
  tls::{
    TlsError, de::De, key_schedule::KeyScheduleState,
    protocol::record_content_type::RecordContentType, tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

create_enum! {
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub(crate) enum AlertDescription<u8> {
    CloseNotify = (0), // Warning
    UnexpectedMessage = (10),
    BadRecordMac = (20),
    RecordOverflow = (22),
    HandshakeFailure = (40),
    BadCertificate = (42),
    UnsupportedCertificate = (43),
    CertificateRevoked = (44),
    CertificateExpired = (45),
    CertificateUnknown = (46),
    IllegalParameter = (47),
    UnknownCa = (48),
    AccessDenied = (49),
    DecodeError = (50),
    DecryptError = (51),
    ProtocolVersion = (70),
    InsufficientSecurity = (71),
    InternalError = (80),
    InappropriateFallback = (86),
    UserCanceled = (90), // Warning
    MissingExtension = (109),
    UnsupportedExtension = (110),
    UnrecognizedName = (112),
    BadCertificateStatusResponse = (113),
    UnknownPskIdentity = (115),
    CertificateRequired = (116),
    NoApplicationProtocol = (120),
  }
}

impl AlertDescription {
  /// Returns `true` if the instance is [`Self::CloseNotify`] or [`Self::UserCanceled`].
  #[must_use]
  pub(crate) const fn is_warning(self) -> bool {
    matches!(self, Self::CloseNotify | Self::UserCanceled)
  }
}

create_enum! {
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub(crate) enum AlertLevel<u8> {
    Warning = (1),
    Fatal = (2),
  }
}

/// Closure information and errors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alert {
  level: AlertLevel,
  description: AlertDescription,
}

impl Alert {
  pub(crate) fn new(level: AlertLevel, description: AlertDescription) -> Self {
    Self { level, description }
  }

  pub(crate) fn data_bytes(self) -> [u8; 2] {
    [u8::from(self.level), u8::from(self.description)]
  }

  pub(crate) fn description(self) -> AlertDescription {
    self.description
  }

  pub(crate) fn record_bytes(
    [a0, a1]: [u8; 2],
    kss: &mut KeyScheduleState,
  ) -> crate::Result<[u8; 5 + 2 + 1 + 16]> {
    let header = [RecordContentType::ApplicationData.into(), 3, 3, 0, 19];
    let mut encrypted = [a0, a1, RecordContentType::Alert.into()];
    let nonce = kss.nonce();
    let secret = kss.cipher_key();
    let tag = kss.cipher_suite().aes_encrypt(&header, &mut encrypted, nonce, secret)?;
    let [b0, b1, b2, b3, b4] = header;
    let [b5, b6, b7] = encrypted;
    let mut rslt = [b0, b1, b2, b3, b4, b5, b6, b7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    if let Some(elem) = rslt.last_chunk_mut::<AEAD_TAG_LEN>() {
      elem.copy_from_slice(&tag);
    }
    kss.increment_counter();
    Ok(rslt)
  }
}

impl<'de> Decode<'de, De> for Alert {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, b1, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidAlert.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self { level: (*b0).try_into()?, description: (*b1).try_into()? })
  }
}

impl Encode<De> for Alert {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&self.data_bytes())?;
    Ok(())
  }
}
