use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::CrlReason,
};

/// Identifies the reason for the certificate revocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReasonCode {
  /// [`CrlReason`]
  pub reason: CrlReason,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for ReasonCode {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self { reason: CrlReason::decode(dw)? })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for ReasonCode {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.reason.encode(ew)
  }
}
