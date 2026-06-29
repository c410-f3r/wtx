use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{GeneralName, X509Error},
};

/// Represents a name range used in name constraints.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneralSubtree<B> {
  /// Defines the subtree root.
  pub base: GeneralName<B>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for GeneralSubtree<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidGeneralSubtree.into());
    };
    dw.bytes = value;
    let base = GeneralName::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { base })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for GeneralSubtree<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      self.base.encode(local_ew)?;
      Ok(())
    })
  }
}
