use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, ENUMERATED_TAG, U32},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};

/// Identifies the reason for the certificate revocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrlReason {
  /// No specific reason provided
  Unspecified,
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
  /// Entry removed from CRL (used in delta CRLs)
  RemoveFromCrl,
  /// Privileges granted to the subject have been withdrawn
  PrivilegeWithdrawn,
  /// Authority attribute (AA) has been compromised
  AaCompromise,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CrlReason {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    dw.decode_aux.tag = Some(ENUMERATED_TAG);
    let rslt = U32::decode(dw)?.u32().try_into();
    dw.decode_aux.tag = None;
    rslt
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for CrlReason {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    ew.encode_aux.tag = Some(ENUMERATED_TAG);
    U32::from_u8((*self).into()).encode(ew)?;
    ew.encode_aux.tag = None;
    Ok(())
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
