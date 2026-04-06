use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Opt, SEQUENCE_TAG, U32, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{GeneralName, MAXIMUM_TAG, MINIMUM_TAG, X509Error},
};

/// Represents a name range used in name constraints.
#[derive(Debug, PartialEq)]
pub struct GeneralSubtree<'bytes> {
  /// Defines the subtree root.
  pub base: GeneralName<'bytes>,
  /// Optional minimum distance.
  pub minimum: Option<U32>,
  /// Optional maximum distance.
  pub maximum: Option<U32>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for GeneralSubtree<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidGeneralSubtree.into());
    };
    dw.bytes = value;
    let base = GeneralName::decode(dw)?;
    let minimum = Opt::decode(dw, MINIMUM_TAG)?.0;
    let maximum = Opt::decode(dw, MAXIMUM_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { base, minimum, maximum })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for GeneralSubtree<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      self.base.encode(local_ew)?;
      Opt(&self.minimum).encode(local_ew, MINIMUM_TAG)?;
      Opt(&self.maximum).encode(local_ew, MAXIMUM_TAG)?;
      Ok(())
    })
  }
}
