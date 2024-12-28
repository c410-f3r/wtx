use crate::tls::TlsError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameType {
  HostName = 0,
}

impl TryFrom<&mut &[u8]> for NameType {
  type Error = crate::Error;

  fn try_from(value: &mut &[u8]) -> Result<Self, Self::Error> {
    let [0, rest @ ..] = value else {
      return Err(TlsError::UnknownNameType.into());
    };
    *value = rest;
    Ok(Self::HostName)
  }
}
