use crate::{
  asn1::{Any, Len, Oid, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// A single attribute type-value pair within a distinguished name.
#[derive(Debug, PartialEq)]
pub struct AttributeTypeAndValue<'bytes> {
  /// See [`Oid`].
  pub oid: Oid,
  /// Opaque bytes
  pub value: Any<&'bytes [u8]>,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for AttributeTypeAndValue<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAttributeTypeAndValue.into());
    };
    dw.bytes = value;
    let oid = Oid::decode(dw)?;
    let value = Any::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { oid, value })
  }
}

impl<'bytes> Encode<GenericCodec<(), ()>> for AttributeTypeAndValue<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE, SEQUENCE_TAG, |local_ew| {
      self.oid.encode(local_ew)?;
      self.value.encode(local_ew)?;
      Ok(())
    })
  }
}
