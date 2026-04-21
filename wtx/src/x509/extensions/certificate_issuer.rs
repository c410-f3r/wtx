use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::GeneralNames,
};

/// Identifies the certificate issuer associated with an entry in an indirect CRL, that is, a
/// CRL that has the indirectCRL indicator set in its issuing distribution point extension.
#[derive(Debug, PartialEq)]
pub struct CertificateIssuer<'bytes> {
  /// See [`GeneralNames`].
  pub general_names: GeneralNames<'bytes>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CertificateIssuer<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self { general_names: GeneralNames::decode(dw)? })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for CertificateIssuer<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.general_names.encode(ew)
  }
}
