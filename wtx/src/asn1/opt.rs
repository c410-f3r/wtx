use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::SingleTypeStorage,
  misc::Lease,
};

/// Optional field helper that delegates operations to the inner element, if any.
#[derive(Debug, Default, PartialEq)]
pub struct Opt<T, const TAG: u8>(
  /// Optional element
  pub T,
);

impl<'de, T, const TAG: u8> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>
  for Opt<Option<T>, TAG>
where
  T: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    if dw.bytes.first().copied() == Some(TAG) {
      Ok(Self(Some(T::decode(dw)?)))
    } else {
      Ok(Self(None))
    }
  }
}

impl<E, T, const TAG: u8> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Opt<T, TAG>
where
  E: Encode<GenericCodec<(), Asn1EncodeWrapperAux>>,
  T: Lease<Option<E>> + SingleTypeStorage<Item = E>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    if let Some(elem) = self.0.lease() {
      elem.encode(ew)?;
    }
    Ok(())
  }
}
