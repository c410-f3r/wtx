use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::{TlsError, de::De},
};

create_enum! {
  #[derive(Clone, Copy, Debug)]
  pub(crate) enum AlertDescription<u8> {
    CloseNotify = (0),
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
    UserCanceled = (90),
    MissingExtension = (109),
    UnsupportedExtension = (110),
    UnrecognizedName = (112),
    BadCertificateStatusResponse = (113),
    UnknownPskIdentity = (115),
    CertificateRequired = (116),
    NoApplicationProtocol = (120),
  }
}

create_enum! {
  #[derive(Debug, Clone, Copy)]
  pub(crate) enum AlertLevel<u8> {
    Warning = (1),
    Fatal = (2),
  }
}

/// Closure information and errors.
#[derive(Debug)]
pub(crate) struct Alert {
  pub(crate) level: AlertLevel,
  pub(crate) description: AlertDescription,
}

impl Alert {
    pub(crate) fn new(level: AlertLevel, description: AlertDescription) -> Self {
        Self { level, description }
    }
}

impl<'de> Decode<'de, De> for Alert {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let [a, b, ref rest @ ..] = **dw else {
      return Err(TlsError::InvalidAlert.into());
    };
    *dw = rest;
    Ok(Self { level: a.try_into()?, description: b.try_into()? })
  }
}

impl<'any> Encode<De> for Alert {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slice(&[u8::from(self.level), u8::from(self.description)])?;
    Ok(())
  }
}
