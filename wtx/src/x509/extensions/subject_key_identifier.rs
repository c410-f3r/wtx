use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::KeyIdentifier,
};

/// Provides a means of identifying certificates that contain a particular public key.
#[derive(Debug, PartialEq)]
pub struct SubjectKeyIdentifier {
  /// See [`KeyIdentifier`].
  pub key_identifier: KeyIdentifier,
}

impl SubjectKeyIdentifier {
  /// Shortcut
  #[inline]
  pub const fn new(key_identifier: KeyIdentifier) -> Self {
    Self { key_identifier }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SubjectKeyIdentifier {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self { key_identifier: KeyIdentifier::decode(dw)? })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for SubjectKeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.key_identifier.encode(ew)
  }
}
