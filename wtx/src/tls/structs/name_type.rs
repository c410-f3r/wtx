use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::{TlsError, de::De},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameType {
  HostName = 0,
}

impl<'de> Decode<'de, De> for NameType {
  #[inline]
  fn decode(_: &mut (), dw: &mut &'de [u8]) -> crate::Result<Self> {
    let [0, rest @ ..] = dw else {
      return Err(TlsError::UnknownNameType.into());
    };
    *dw = rest;
    Ok(Self::HostName)
  }
}

impl Encode<De> for NameType {
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_byte(0)?;
    Ok(())
  }
}
