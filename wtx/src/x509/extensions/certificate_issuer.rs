use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::GeneralNames,
};

/// Identifies the certificate issuer associated with an entry in an indirect CRL, that is, a
/// CRL that has the indirectCRL indicator set in its issuing distribution point extension.
#[derive(Debug, PartialEq)]
pub struct CertificateIssuer<B> {
  /// See [`GeneralNames`].
  pub general_names: GeneralNames<B>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for CertificateIssuer<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self { general_names: GeneralNames::decode(dw)? })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for CertificateIssuer<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.general_names.encode(ew)
  }
}
