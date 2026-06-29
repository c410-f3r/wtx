use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::CrlReason,
};

/// Identifies the reason for the certificate revocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReasonCode {
  /// [`CrlReason`]
  pub reason: CrlReason,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for ReasonCode {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self { reason: CrlReason::decode(dw)? })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for ReasonCode {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.reason.encode(ew)
  }
}
