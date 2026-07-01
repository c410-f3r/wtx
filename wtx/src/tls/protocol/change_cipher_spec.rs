use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

/// Used because of compatibility concerns. Does not have any semantical value in TLS 1.3.
pub(crate) struct ChangeCipherSpec {}

impl<'de> Decode<'de, De> for ChangeCipherSpec {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [1, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self {})
  }
}

impl Encode<De> for ChangeCipherSpec {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(1)
  }
}
