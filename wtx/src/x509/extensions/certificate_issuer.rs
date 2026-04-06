use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::GeneralNames,
};

/// Identifies the certificate issuer associated with an entry in an indirect CRL, that is, a
/// CRL that has the indirectCRL indicator set in its issuing distribution point extension.
#[derive(Debug, PartialEq)]
pub struct CertificateIssuer<'bytes>(
  /// See [`GeneralNames`].
  pub GeneralNames<'bytes>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CertificateIssuer<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(GeneralNames::decode(dw)?))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for CertificateIssuer<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
