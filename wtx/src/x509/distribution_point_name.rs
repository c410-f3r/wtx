use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{
    DISTRIBUTION_POINT_NAME_FULL_NAME_TAG, DISTRIBUTION_POINT_NAME_RELATIVE_TAG, GeneralNames,
    RelativeDistinguishedName, X509Error,
  },
};

/// Specifies where to find CRL information for a certificate.
#[derive(Clone, Debug, PartialEq)]
pub enum DistributionPointName<B> {
  /// [`GeneralNames`].
  FullName(GeneralNames<B>),
  /// [`RelativeDistinguishedName`].
  NameRelativeToCrlIssuer(RelativeDistinguishedName<B>),
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for DistributionPointName<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    match dw.bytes.first().copied() {
      Some(DISTRIBUTION_POINT_NAME_FULL_NAME_TAG) => {
        dw.decode_aux.tag = Some(DISTRIBUTION_POINT_NAME_FULL_NAME_TAG);
        let general_names = GeneralNames::decode(dw)?;
        dw.decode_aux.tag = None;
        Ok(Self::FullName(general_names))
      }
      Some(DISTRIBUTION_POINT_NAME_RELATIVE_TAG) => {
        Ok(Self::NameRelativeToCrlIssuer(RelativeDistinguishedName {
          entries: SequenceBuffer::decode(dw, DISTRIBUTION_POINT_NAME_RELATIVE_TAG)?.0.0,
        }))
      }
      _ => Err(X509Error::InvalidExtensionCrlDistributionPoints.into()),
    }
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for DistributionPointName<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    match self {
      Self::FullName(elem) => SequenceBuffer(&elem.entries).encode(
        ew,
        Len::MAX_ONE_BYTE,
        DISTRIBUTION_POINT_NAME_FULL_NAME_TAG,
      ),
      Self::NameRelativeToCrlIssuer(elem) => SequenceBuffer(&elem.entries).encode(
        ew,
        Len::MAX_ONE_BYTE,
        DISTRIBUTION_POINT_NAME_RELATIVE_TAG,
      ),
    }
  }
}
