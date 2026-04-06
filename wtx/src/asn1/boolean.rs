use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, BOOLEAN_TAG, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// `true` or `false`
#[derive(Debug, PartialEq)]
pub struct Boolean(
  /// Value
  pub bool,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Boolean {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (BOOLEAN_TAG, _, [boolean], rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidBoolean.into());
    };
    dw.bytes = rest;
    Ok(Self(*boolean != 0))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for Boolean {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[BOOLEAN_TAG][..],
      &*Len::from_usize(0, 1)?,
      &[if self.0 { 255 } else { 0 }],
    ])?;
    Ok(())
  }
}
