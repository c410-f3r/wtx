use crate::{
  asn1::{Asn1Error, BOOLEAN_TAG, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// `true` or `false`
#[derive(Debug, PartialEq)]
pub struct Boolean(
  /// Value
  pub bool,
);

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Boolean {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (BOOLEAN_TAG, _, [boolean], rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidBoolean.into());
    };
    dw.bytes = rest;
    Ok(Self(*boolean != 0))
  }
}

impl Encode<GenericCodec<(), ()>> for Boolean {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[BOOLEAN_TAG][..],
      &*Len::from_usize(0, 1)?,
      &[if self.0 { 1 } else { 255 }],
    ])?;
    Ok(())
  }
}
