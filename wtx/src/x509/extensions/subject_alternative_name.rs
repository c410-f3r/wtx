use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{GeneralNames, X509Error},
};

/// Allows identities to be bound to the subject of the certificate.
#[derive(Clone, Debug, PartialEq)]
pub struct SubjectAlternativeName<B> {
  /// See [`GeneralNames`]
  pub general_names: GeneralNames<B>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for SubjectAlternativeName<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let general_names = GeneralNames::<B>::decode(dw).map_err(|_err| X509Error::InvalidSan)?;
    for elem in &general_names.entries {
      let (_tag, bytes) = elem.into();
      if bytes.lease().contains(&b'_') {
        return Err(X509Error::InvalidSan.into());
      }
    }
    Ok(Self { general_names })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for SubjectAlternativeName<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.general_names.encode(ew)
  }
}
