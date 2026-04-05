use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, U32},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReasonFlags {
  Unused,
  KeyCompromise,
  CaCompromise,
  AffiliationChanged,
  Superseded,
  CessationOfOperation,
  CertificateHold,
  PrivilegeWithdrawn,
  AaCompromise,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for ReasonFlags {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    U32::decode(dw)?.u32().try_into()
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for ReasonFlags {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    U32::from_u8((*self).into()).encode(ew)
  }
}

impl From<ReasonFlags> for u8 {
  #[inline]
  fn from(value: ReasonFlags) -> Self {
    match value {
      ReasonFlags::Unused => 0,
      ReasonFlags::KeyCompromise => 1,
      ReasonFlags::CaCompromise => 2,
      ReasonFlags::AffiliationChanged => 3,
      ReasonFlags::Superseded => 4,
      ReasonFlags::CessationOfOperation => 5,
      ReasonFlags::CertificateHold => 6,
      ReasonFlags::PrivilegeWithdrawn => 7,
      ReasonFlags::AaCompromise => 8,
    }
  }
}

impl TryFrom<u32> for ReasonFlags {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    Ok(match value {
      0 => ReasonFlags::Unused,
      1 => ReasonFlags::KeyCompromise,
      2 => ReasonFlags::CaCompromise,
      3 => ReasonFlags::AffiliationChanged,
      4 => ReasonFlags::Superseded,
      5 => ReasonFlags::CessationOfOperation,
      6 => ReasonFlags::CertificateHold,
      7 => ReasonFlags::PrivilegeWithdrawn,
      8 => ReasonFlags::AaCompromise,
      _ => return Err(X509Error::InvalidExtensionReasonCode.into()),
    })
  }
}
