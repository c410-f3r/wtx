use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SET_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorU8,
  misc::Lease,
  x509::AttributeTypeAndValue,
};

/// Unordered set of attribute type-value pairs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelativeDistinguishedName<B> {
  /// Entries
  pub entries: ArrayVectorU8<AttributeTypeAndValue<B>, 2>,
}

impl<B> RelativeDistinguishedName<B> {
  /// Shortcut
  #[inline]
  pub const fn new(entries: ArrayVectorU8<AttributeTypeAndValue<B>, 2>) -> Self {
    Self { entries }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for RelativeDistinguishedName<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self { entries: SequenceBuffer::decode(dw, SET_TAG)?.0.0 })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for RelativeDistinguishedName<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.entries).encode(ew, Len::MAX_ONE_BYTE, SET_TAG)
  }
}
