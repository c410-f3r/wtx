use crate::tls::{AlertDescription, TlsError};

/// <https://datatracker.ietf.org/doc/html/rfc9846#appendix-B.3>
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum HandshakeTy {
  ClientHello = 1,
  ServerHello = 2,
  NewSessionTicket = 4,
  EndOfEarlyData = 5,
  EncryptedExtensions = 8,
  Certificate = 11,
  CertificateRequest = 13,
  CertificateVerify = 15,
  Finished = 20,
  KeyUpdate = 24,
  MessageHash = 254,
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
          TlsError::UnknownHandshakeTy,
          AlertDescription::UnexpectedMessage,
        ));
      }
    })
  }
}
