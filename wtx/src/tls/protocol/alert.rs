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
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  /// Alert Description
  pub enum AlertDescription<u8> {
    /// Close Notify
    CloseNotify = (0),
    /// Unexpected Message
    UnexpectedMessage = (10),
    /// Bad Record Mac
    BadRecordMac = (20),
    /// Record Overflow
    RecordOverflow = (22),
    /// Handshake Failure
    HandshakeFailure = (40),
    /// Bad Certificate
    BadCertificate = (42),
    /// Unsupported Certificate
    UnsupportedCertificate = (43),
    /// Certificate Revoked
    CertificateRevoked = (44),
    /// Certificate Expired
    CertificateExpired = (45),
    /// Certificate Unknown
    CertificateUnknown = (46),
    /// Illegal Parameter
    IllegalParameter = (47),
    /// Unknown Ca
    UnknownCa = (48),
    /// Access Denied
    AccessDenied = (49),
    /// Decode Error
    DecodeError = (50),
    /// Decrypt Error
    DecryptError = (51),
    /// Protocol Version
    ProtocolVersion = (70),
    /// Insufficient Security
    InsufficientSecurity = (71),
    /// Internal Error
    InternalError = (80),
    /// Inappropriate Fallback
    InappropriateFallback = (86),
    /// User Canceled
    UserCanceled = (90),
    /// Missing Extension
    MissingExtension = (109),
    /// Unsupported Extension
    UnsupportedExtension = (110),
    /// Unrecognize dName
    UnrecognizedName = (112),
    /// Bad Certificate Status Response
    BadCertificateStatusResponse = (113),
    /// Unknown Psk Identity
    UnknownPskIdentity = (115),
    /// Certificate Required
    CertificateRequired = (116),
    /// No Application Protocol
    NoApplicationProtocol = (120),
  }
}

create_enum! {
  /// Alert level
  #[derive(Debug, Clone, Copy, Eq, PartialEq)]
  pub enum AlertLevel<u8> {
    /// Warning
    Warning = (1),
    /// Fatal
    Fatal = (2),
  }
}

/// Closure information and errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Alert {
  level: AlertLevel,
  description: AlertDescription,
}

impl Alert {
  pub(crate) fn fatal(description: AlertDescription) -> Self {
    Self { level: AlertLevel::Fatal, description }
  }

  pub(crate) const fn close_notify() -> Self {
    Self { level: AlertLevel::Warning, description: AlertDescription::CloseNotify }
  }

  /// <https://datatracker.ietf.org/doc/html/rfc9846#section-6.1>
  ///
  /// `user_canceled` is a nice-to-have but optional thing that this implementation chose to
  /// ignore. All parties must send a `close_notify`, regardless if `user_canceled` was or was
  /// not sent before.
  ///
  /// Besides, `user_canceled` doesn't require it to be replied back to the sender.
  pub(crate) fn is_close_notify(self) -> bool {
    matches!((self.description, self.level), (AlertDescription::CloseNotify, AlertLevel::Warning))
  }

  pub(crate) fn record_bytes(
    self,
    kss: &mut KeyScheduleState,
  ) -> crate::Result<[u8; 5 + 2 + 1 + 16]> {
    let [a0, a1] = self.data_bytes();
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

  pub(crate) fn record_bytes_unencrypted(self) -> [u8; 5 + 2] {
    let [a0, a1] = self.data_bytes();
    [RecordContentType::ApplicationData.into(), 3, 3, 0, 2, a0, a1]
  }

  fn data_bytes(self) -> [u8; 2] {
    [u8::from(self.level), u8::from(self.description)]
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
