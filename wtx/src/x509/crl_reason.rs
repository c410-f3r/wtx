use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, U32},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrlReason {
  Unspecified,
  KeyCompromise,
  CaCompromise,
  AffiliationChanged,
  Superseded,
  CessationOfOperation,
  CertificateHold,
  RemoveFromCrl,
  PrivilegeWithdrawn,
  AaCompromise,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CrlReason {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    U32::decode(dw)?.u32().try_into()
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for CrlReason {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    U32::from_u8((*self).into()).encode(ew)
  }
}

impl From<CrlReason> for u8 {
  #[inline]
  fn from(value: CrlReason) -> Self {
    match value {
      CrlReason::Unspecified => 0,
      CrlReason::KeyCompromise => 1,
      CrlReason::CaCompromise => 2,
      CrlReason::AffiliationChanged => 3,
      CrlReason::Superseded => 4,
      CrlReason::CessationOfOperation => 5,
      CrlReason::CertificateHold => 6,
      CrlReason::RemoveFromCrl => 8,
      CrlReason::PrivilegeWithdrawn => 9,
      CrlReason::AaCompromise => 10,
    }
  }
}

impl TryFrom<u32> for CrlReason {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    Ok(match value {
      0 => CrlReason::Unspecified,
      1 => CrlReason::KeyCompromise,
      2 => CrlReason::CaCompromise,
      3 => CrlReason::AffiliationChanged,
      4 => CrlReason::Superseded,
      5 => CrlReason::CessationOfOperation,
      6 => CrlReason::CertificateHold,
      8 => CrlReason::RemoveFromCrl,
      9 => CrlReason::PrivilegeWithdrawn,
      10 => CrlReason::AaCompromise,
      _ => return Err(X509Error::InvalidExtensionReasonCode.into()),
    })
  }
}
