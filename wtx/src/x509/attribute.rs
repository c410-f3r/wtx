use crate::{
  asn1::{
    Any, Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, Oid, SEQUENCE_TAG, SET_TAG,
    SequenceBuffer, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorU8,
  misc::Lease,
  x509::X509Error,
};

/// Generalization of `AttributeTypeAndValue`.
#[derive(Debug, PartialEq)]
pub struct Attribute<B, const N: usize> {
  /// See [`Oid`].
  pub oid: Oid,
  /// Collection of opaque bytes
  pub value: ArrayVectorU8<Any<B>, N>,
}

impl<'de, B, const N: usize> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Attribute<B, N>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAttribute.into());
    };
    dw.bytes = value;
    let oid = Oid::decode(dw)?;
    let vector = SequenceBuffer::decode(dw, SET_TAG)?.0.0;
    dw.bytes = rest;
    Ok(Self { oid, value: vector })
  }
}

impl<B, const N: usize> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Attribute<B, N>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.oid.encode(local_ew)?;
      SequenceBuffer(&self.value).encode(local_ew, Len::MAX_ONE_BYTE, SET_TAG)?;
      Ok(())
    })
  }
}
