use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{
    DISTRIBUTION_POINT_NAME_FULL_NAME_TAG, DISTRIBUTION_POINT_NAME_RELATIVE_TAG, GeneralNames,
    RelativeDistinguishedName, X509Error,
  },
};

#[derive(Debug, PartialEq)]
pub enum DistributionPointName<'bytes> {
  FullName(GeneralNames<'bytes>),
  NameRelativeToCrlIssuer(RelativeDistinguishedName<'bytes>),
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for DistributionPointName<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    dw.bytes = value;
    let result = match tag {
      DISTRIBUTION_POINT_NAME_FULL_NAME_TAG => Self::FullName(GeneralNames::decode(dw)?),
      DISTRIBUTION_POINT_NAME_RELATIVE_TAG => {
        Self::NameRelativeToCrlIssuer(RelativeDistinguishedName::decode(dw)?)
      }
      _ => return Err(X509Error::InvalidExtensionCrlDistributionPoints.into()),
    };
    dw.bytes = rest;
    Ok(result)
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for DistributionPointName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    match self {
      Self::FullName(elem) => elem.encode(ew),
      Self::NameRelativeToCrlIssuer(elem) => elem.encode(ew),
    }
  }
}
