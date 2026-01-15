use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::{TlsError, de::De},
};

create_enum! {
  /// Without this extension, TLS specifies a fixed maximum plaintext
  /// fragment length of 2^14 bytes. It may be desirable for constrained
  /// clients to negotiate a smaller maximum fragment length due to memory
  /// limitations or bandwidth limitations.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum MaxFragmentLength<u8> {
    /// 512 bytes
    _512 = (1),
    /// 1024 bytes
    _1024 = (2),
    /// 2048 bytes
    _2048 = (3),
    /// 4096 bytes
    _4096 = (4),
  }
}

impl<'de> Decode<'de, De> for MaxFragmentLength {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let [a, rest @ ..] = dw else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw = rest;
    Ok(Self::try_from(*a)?)
  }
}

impl Encode<De> for MaxFragmentLength {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_byte(u8::from(*self))
  }
}
