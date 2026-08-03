use crate::tls::{AlertDescription, TlsError};

/// <https://datatracker.ietf.org/doc/html/rfc9846#appendix-B.3>
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HandshakeTy {
  /// Client Hello
  ClientHello = 1,
  /// Server Hello
  ServerHello = 2,
  /// New Session Ticket
  NewSessionTicket = 4,
  /// End Of Early Data
  EndOfEarlyData = 5,
  /// Encrypted Extensions
  EncryptedExtensions = 8,
  /// Certificate
  Certificate = 11,
  /// Certificate Request
  CertificateRequest = 13,
  /// Certificate Verify
  CertificateVerify = 15,
  /// Finished
  Finished = 20,
  /// Key Update
  KeyUpdate = 24,
  /// Message Hash
  MessageHash = 254,
}

impl HandshakeTy {
  /// Returns `true` if this instance [`HandshakeTy::Finished`].
  #[inline]
  #[must_use]
  pub const fn is_finished(&self) -> bool {
    matches!(self, Self::Finished)
  }
}

impl From<HandshakeTy> for u8 {
  #[inline]
  fn from(from: HandshakeTy) -> Self {
    match from {
      HandshakeTy::ClientHello => 1,
      HandshakeTy::ServerHello => 2,
      HandshakeTy::NewSessionTicket => 4,
      HandshakeTy::EndOfEarlyData => 5,
      HandshakeTy::EncryptedExtensions => 8,
      HandshakeTy::Certificate => 11,
      HandshakeTy::CertificateRequest => 13,
      HandshakeTy::CertificateVerify => 15,
      HandshakeTy::Finished => 20,
      HandshakeTy::KeyUpdate => 24,
      HandshakeTy::MessageHash => 254,
    }
  }
}

impl TryFrom<u8> for HandshakeTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> crate::Result<Self> {
    Ok(match from {
      1 => Self::ClientHello,
      2 => Self::ServerHello,
      4 => Self::NewSessionTicket,
      5 => Self::EndOfEarlyData,
      8 => Self::EncryptedExtensions,
      11 => Self::Certificate,
      13 => Self::CertificateRequest,
      15 => Self::CertificateVerify,
      20 => Self::Finished,
      24 => Self::KeyUpdate,
      254 => Self::MessageHash,
      _ => {
        return Err(crate::Error::TlsErrorReply(
          TlsError::UnknownHandshakeTy(from),
          AlertDescription::UnexpectedMessage,
        ));
      }
    })
  }
}
