use crate::{
  asn1::{
    Any, Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Oid, SEQUENCE_TAG, SET_TAG, SequenceBuffer,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::ArrayVectorU8,
  x509::X509Error,
};

/// Generalization of `AttributeTypeAndValue`.
#[derive(Debug, PartialEq)]
pub struct Attribute<'bytes, const N: usize> {
  /// See [`Oid`].
  pub oid: Oid,
  /// Collection of opaque bytes
  pub value: ArrayVectorU8<Any<&'bytes [u8]>, N>,
}

impl<'de, const N: usize> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Attribute<'de, N> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAttribute.into());
    };
    dw.bytes = value;
    let oid = Oid::decode(dw)?;
    let value = SequenceBuffer::decode(dw, SET_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { oid, value })
  }
}

impl<'bytes, const N: usize> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Attribute<'bytes, N> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.oid.encode(local_ew)?;
      SequenceBuffer(&self.value).encode(local_ew, Len::MAX_ONE_BYTE, SET_TAG)?;
      Ok(())
    })
  }
}
