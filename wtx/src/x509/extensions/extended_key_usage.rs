use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Oid, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
};

/// This extension indicates one or more purposes for which the certified public key may be used,
/// in addition to or in place of the basic purposes indicated in the key usage extension.
#[derive(Debug, PartialEq)]
pub struct ExtendedKeyUsage(Vector<Oid>);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for ExtendedKeyUsage {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for ExtendedKeyUsage {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)?;
    Ok(())
  }
}
