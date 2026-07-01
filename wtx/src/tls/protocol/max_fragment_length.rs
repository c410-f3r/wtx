use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
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

impl MaxFragmentLength {
  /// Numerical representation
  #[inline]
  pub const fn num(&self) -> u16 {
    match self {
      MaxFragmentLength::_512 => 512,
      MaxFragmentLength::_1024 => 1024,
      MaxFragmentLength::_2048 => 2048,
      MaxFragmentLength::_4096 => 4096,
    }
  }
}

impl<'de> Decode<'de, De> for MaxFragmentLength {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw.bytes_mut() = rest;
    Self::try_from(*b0)
  }
}

impl Encode<De> for MaxFragmentLength {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(u8::from(*self))
  }
}
