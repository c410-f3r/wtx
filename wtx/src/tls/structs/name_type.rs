use crate::{
  misc::{Decode, Encode, SuffixWriterMut},
  tls::{TlsError, de::De},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameType {
  HostName = 0,
}

impl Decode<'_, De> for NameType {
  #[inline]
  fn decode(dw: &mut &[u8]) -> crate::Result<Self> {
    let [0, rest @ ..] = dw else {
      return Err(TlsError::UnknownNameType.into());
    };
    *dw = rest;
    Ok(Self::HostName)
  }
}

impl Encode<De> for NameType {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew._extend_from_byte(0)?;
    Ok(())
  }
}
