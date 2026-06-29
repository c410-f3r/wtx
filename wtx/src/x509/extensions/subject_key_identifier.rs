use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::KeyIdentifier,
};

/// Provides a means of identifying certificates that contain a particular public key.
#[derive(Clone, Debug, PartialEq)]
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

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for SubjectKeyIdentifier {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self { key_identifier: KeyIdentifier::decode(dw)? })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for SubjectKeyIdentifier {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.key_identifier.encode(ew)
  }
}
