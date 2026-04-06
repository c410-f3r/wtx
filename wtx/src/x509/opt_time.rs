use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, GENERALIZED_TIME_TAG, UTC_TIME_TAG},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::Time,
};

/// This structure should be used instead of `Opt<Time>`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptTime(
  /// Optional time
  pub Option<Time>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for OptTime {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    if let Some(UTC_TIME_TAG) | Some(GENERALIZED_TIME_TAG) = dw.bytes.first().copied() {
      Ok(Self(Some(Time::decode(dw)?)))
    } else {
      Ok(Self(None))
    }
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for OptTime {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    if let Some(elem) = &self.0 {
      elem.encode(ew)?;
    }
    Ok(())
  }
}
