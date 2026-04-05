use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, U32},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// A `BIT STRING` variation of `ReasonCode`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReasonFlags {
  /// Not used
  Unused,
  /// Subject's private key has been compromised
  KeyCompromise,
  /// Issuing CA's private key has been compromised
  CaCompromise,
  /// Subject is no longer affiliated with the issuing organization
  AffiliationChanged,
  /// Certificate has been replaced by a new one
  Superseded,
  /// Subject has ceased operation
  CessationOfOperation,
  /// Certificate is temporarily on hold
  CertificateHold,
  /// Privileges granted to the subject have been withdrawn
  PrivilegeWithdrawn,
  /// Authority attribute (AA) has been compromised
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
