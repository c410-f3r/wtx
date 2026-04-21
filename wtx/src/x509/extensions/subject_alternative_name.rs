use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{GeneralNames, X509Error},
};

/// Allows identities to be bound to the subject of the certificate.
#[derive(Debug, PartialEq)]
pub struct SubjectAlternativeName<'bytes> {
  /// See [`GeneralNames`]
  pub general_names: GeneralNames<'bytes>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SubjectAlternativeName<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let general_names = GeneralNames::decode(dw).map_err(|_err| X509Error::InvalidSan)?;
    for elem in &general_names.entries {
      let (_tag, bytes) = elem.into();
      if bytes.contains(&b'_') {
        return Err(X509Error::InvalidSan.into());
      }
    }
    Ok(Self { general_names })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for SubjectAlternativeName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.general_names.encode(ew)
  }
}
