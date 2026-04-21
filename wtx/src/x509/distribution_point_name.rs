use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{
    DISTRIBUTION_POINT_NAME_FULL_NAME_TAG, DISTRIBUTION_POINT_NAME_RELATIVE_TAG, GeneralNames,
    RelativeDistinguishedName, X509Error,
  },
};

/// Specifies where to find CRL information for a certificate.
#[derive(Debug, PartialEq)]
pub enum DistributionPointName<'bytes> {
  /// [`GeneralNames`].
  FullName(GeneralNames<'bytes>),
  /// [`RelativeDistinguishedName`].
  NameRelativeToCrlIssuer(RelativeDistinguishedName<'bytes>),
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for DistributionPointName<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    match dw.bytes.first().copied() {
      Some(DISTRIBUTION_POINT_NAME_FULL_NAME_TAG) => {
        dw.decode_aux.tag = Some(DISTRIBUTION_POINT_NAME_FULL_NAME_TAG);
        let general_names = GeneralNames::decode(dw)?;
        dw.decode_aux.tag = None;
        Ok(Self::FullName(general_names))
      }
      Some(DISTRIBUTION_POINT_NAME_RELATIVE_TAG) => {
        Ok(Self::NameRelativeToCrlIssuer(RelativeDistinguishedName {
          entries: SequenceBuffer::decode(dw, DISTRIBUTION_POINT_NAME_RELATIVE_TAG)?.0,
        }))
      }
      _ => Err(X509Error::InvalidExtensionCrlDistributionPoints.into()),
    }
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for DistributionPointName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
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
