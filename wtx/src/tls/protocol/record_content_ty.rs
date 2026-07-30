use crate::tls::{AlertDescription, TlsError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RecordContentTy {
  ChangeCipherSpec = 20,
  Alert = 21,
  Handshake = 22,
  ApplicationData = 23,
}

impl From<RecordContentTy> for u8 {
  #[inline]
  fn from(from: RecordContentTy) -> Self {
    match from {
      RecordContentTy::ChangeCipherSpec => 20,
      RecordContentTy::Alert => 21,
      RecordContentTy::Handshake => 22,
      RecordContentTy::ApplicationData => 23,
    }
  }
}

impl TryFrom<u8> for RecordContentTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> crate::Result<Self> {
    Ok(match from {
      20 => RecordContentTy::ChangeCipherSpec,
      21 => RecordContentTy::Alert,
      22 => RecordContentTy::Handshake,
      23 => RecordContentTy::ApplicationData,
      _ => {
        return Err(crate::Error::TlsErrorReply(
          TlsError::UnknownRecordContentType,
          AlertDescription::UnexpectedMessage,
        ));
      }
    })
  }
}
