use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, BOOLEAN_TAG, Len, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
};

/// `true` or `false`
#[derive(Debug, PartialEq)]
pub struct Boolean(
  /// Value
  pub bool,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Boolean {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (BOOLEAN_TAG, _, [boolean], rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidBoolean.into());
    };
    dw.bytes = rest;
    Ok(Self(*boolean != 0))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Boolean {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[BOOLEAN_TAG][..],
      &*Len::from_usize(0, 1)?,
      &[if self.0 { 255 } else { 0 }],
    ])?;
    Ok(())
  }
}
