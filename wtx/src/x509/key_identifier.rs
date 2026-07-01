use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Octetstring},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorCopy,
};

/// The value is typically a hash of the subject's public key.
//
// `RFC-7093` states a bunch of different encodings but all of them have 20 bytes, as such, it
// is an upper bound.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyIdentifier(ArrayVectorCopy<u8, 20>);

impl KeyIdentifier {
  /// Only allows up to 20 bytes
  #[inline]
  pub const fn new(array_vector: ArrayVectorCopy<u8, 20>) -> Self {
    Self(array_vector)
  }

  /// Internal bytes
  #[inline]
  pub const fn bytes(&self) -> &ArrayVectorCopy<u8, 20> {
    &self.0
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for KeyIdentifier {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(ArrayVectorCopy::from_copyable_slice(Octetstring::<&[u8]>::decode(dw)?.bytes())?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for KeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    Octetstring::from_bytes(&self.0).encode(ew)?;
    Ok(())
  }
}
