use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, asn1_writer, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
};

/// Explicit tags imply in an additional layer of indirection
#[derive(Debug)]
pub struct ExplicitTag<T, const TAG: u8>(
  /// Arbitrary element
  pub T,
);

impl<'de, T, const TAG: u8> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>
  for ExplicitTag<T, TAG>
where
  T: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if tag != TAG {
      return Err(Asn1Error::InvalidExplicitTag.into());
    }
    dw.bytes = value;
    let rslt = T::decode(dw)?;
    dw.bytes = rest;
    Ok(Self(rslt))
  }
}

impl<T, const TAG: u8> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for ExplicitTag<T, TAG>
where
  T: Encode<GenericCodec<(), Asn1EncodeWrapperAux>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, ew.encode_aux.len_guess, TAG, |local_ew| self.0.encode(local_ew))
  }
}
