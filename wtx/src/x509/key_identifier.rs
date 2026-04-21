use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Octetstring},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::ArrayVectorU8,
};

/// The value is typically a hash of the subject's public key.
//
// `RFC-7093` states a bunch of different encodings but all of them have 20 bytes, as such, it
// is an upper bound.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyIdentifier(ArrayVectorU8<u8, 20>);

impl KeyIdentifier {
  /// Only allows up to 20 bytes
  #[inline]
  pub const fn new(array_vector: ArrayVectorU8<u8, 20>) -> Self {
    Self(array_vector)
  }

  /// Internal bytes
  #[inline]
  pub const fn bytes(&self) -> &ArrayVectorU8<u8, 20> {
    &self.0
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for KeyIdentifier {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(ArrayVectorU8::from_copyable_slice(Octetstring::decode(dw)?.bytes())?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for KeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    Octetstring::from_bytes(&self.0).encode(ew)?;
    Ok(())
  }
}
