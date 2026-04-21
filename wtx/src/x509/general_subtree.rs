use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{GeneralName, X509Error},
};

/// Represents a name range used in name constraints.
#[derive(Debug, PartialEq)]
pub struct GeneralSubtree<'bytes> {
  /// Defines the subtree root.
  pub base: GeneralName<'bytes>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for GeneralSubtree<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidGeneralSubtree.into());
    };
    dw.bytes = value;
    let base = GeneralName::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { base })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for GeneralSubtree<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      self.base.encode(local_ew)?;
      Ok(())
    })
  }
}
