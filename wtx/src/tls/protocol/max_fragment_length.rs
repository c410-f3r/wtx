use crate::{
  de::{Decode, Encode},
  tls::{TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
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
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [a, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self::try_from(*a)?)
  }
}

impl Encode<De> for MaxFragmentLength {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_byte(u8::from(*self))
  }
}
