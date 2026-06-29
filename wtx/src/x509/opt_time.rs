use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, GENERALIZED_TIME_TAG, UTC_TIME_TAG},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::Time,
};

/// This structure should be used instead of `Opt<Time>`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptTime(
  /// Optional time
  pub Option<Time>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for OptTime {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    if let Some(GENERALIZED_TIME_TAG | UTC_TIME_TAG) = dw.bytes.first().copied() {
      Ok(Self(Some(Time::decode(dw)?)))
    } else {
      Ok(Self(None))
    }
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for OptTime {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    if let Some(elem) = &self.0 {
      elem.encode(ew)?;
    }
    Ok(())
  }
}
