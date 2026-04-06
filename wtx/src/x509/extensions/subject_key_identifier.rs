use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::KeyIdentifier,
};

/// Provides a means of identifying certificates that contain a particular public key.
#[derive(Debug, PartialEq)]
pub struct SubjectKeyIdentifier(
  /// See [`KeyIdentifier`].
  pub KeyIdentifier,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SubjectKeyIdentifier {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(KeyIdentifier::decode(dw)?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for SubjectKeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
