use crate::{
  asn1::{
    Any, Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, Oid, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::X509Error,
};

/// A single attribute type-value pair within a distinguished name.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeTypeAndValue<B> {
  /// See [`Oid`].
  pub oid: Oid,
  /// Opaque bytes
  pub value: Any<B>,
}

impl<B> AttributeTypeAndValue<B> {
  /// Shortcut
  #[inline]
  pub const fn new(oid: Oid, value: Any<B>) -> Self {
    Self { oid, value }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for AttributeTypeAndValue<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAttributeTypeAndValue.into());
    };
    dw.bytes = value;
    let oid = Oid::decode(dw)?;
    let any = Any::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { oid, value: any })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for AttributeTypeAndValue<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.oid.encode(local_ew)?;
      self.value.encode(local_ew)?;
      Ok(())
    })
  }
}
