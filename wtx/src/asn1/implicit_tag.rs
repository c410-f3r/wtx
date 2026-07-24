use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
};

/// Implicity tags just override the default tags of existing elements
#[derive(Debug)]
pub struct ImplicitTag<T, const TAG: u8>(
  /// Arbitrary element
  pub T,
);

impl<'de, T, const TAG: u8> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>
  for ImplicitTag<T, TAG>
where
  T: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    dw.decode_aux.tag = Some(TAG);
    let rslt = T::decode(dw);
    dw.decode_aux.tag = None;
    Ok(Self(rslt?))
  }
}

impl<T, const TAG: u8> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for ImplicitTag<T, TAG>
where
  T: Encode<GenericCodec<(), Asn1EncodeWrapperAux>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    ew.encode_aux.tag = Some(TAG);
    let rslt = self.0.encode(ew);
    ew.encode_aux.tag = None;
    rslt
  }
}
