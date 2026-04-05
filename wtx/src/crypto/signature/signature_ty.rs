use crate::crypto::{GlobalEd25519, GlobalP256Signature, GlobalP384Signature, Signature};
#[cfg(feature = "asn1")]
use crate::{
  asn1::{OID_EC_P256, OID_KEY_TYPE_EC_PUBLIC_KEY, OID_NIST_EC_P384, OID_SIG_ED25519, Oid},
  crypto::CryptoError,
};
#[cfg(feature = "database")]
use crate::{
  codec::{CodecController, Decode, Encode},
  database::{Database, Typed},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

create_enum! {
  /// Specifies the group or curve used for agreements.
  #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
  pub enum SignatureTy<u16> {
    /// Ed25519
    Ed25519 = (1),
    /// P256
    P256 = (2),
    /// P384
    P384 = (3),
  }
}

impl SignatureTy {
  /// Calls [`Signature::validate`] according to the current instance value and the selected crypto
  /// backend.
  #[inline]
  pub fn validate_signature(&self, pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    match self {
      SignatureTy::Ed25519 => GlobalEd25519::validate(pk, msg, signature),
      SignatureTy::P256 => GlobalP256Signature::validate(pk, msg, signature),
      SignatureTy::P384 => GlobalP384Signature::validate(pk, msg, signature),
    }
  }
}

#[cfg(feature = "database")]
impl<'de, CC> Decode<'de, CC> for SignatureTy
where
  CC: CodecController,
  &'de str: Decode<'de, CC>,
{
  #[inline]
  fn decode(input: &mut CC::DecodeWrapper<'de, '_, '_>) -> Result<Self, CC::Error> {
    let string: &str = Decode::<'de, CC>::decode(input)?;
    Ok(Self::try_from(string.as_bytes())?)
  }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SignatureTy {
  #[inline]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: &str = Deserialize::deserialize(deserializer)?;
    Self::try_from(s.as_bytes()).map_err(serde::de::Error::custom)
  }
}

#[cfg(feature = "database")]
impl<CC> Encode<CC> for SignatureTy
where
  CC: CodecController,
  for<'any> &'any str: Encode<CC>,
{
  #[inline]
  fn encode(&self, ew: &mut CC::EncodeWrapper<'_, '_, '_>) -> Result<(), CC::Error> {
    <&str as Encode<CC>>::encode(&(*self).into(), ew)
  }
}

#[cfg(feature = "serde")]
impl Serialize for SignatureTy {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(<&str>::from(*self))
  }
}

#[cfg(feature = "database")]
impl<D> Typed<D> for SignatureTy
where
  D: Database,
  for<'any> &'any str: Typed<D>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<D::Ty> {
    <&str as Typed<D>>::runtime_ty(&<&str>::from(*self))
  }

  #[inline]
  fn static_ty() -> Option<D::Ty> {
    <&str as Typed<D>>::static_ty()
  }
}

impl From<SignatureTy> for &'static str {
  #[inline]
  fn from(from: SignatureTy) -> Self {
    match from {
      SignatureTy::Ed25519 => "ED25519",
      SignatureTy::P256 => "P256",
      SignatureTy::P384 => "P384",
    }
  }
}

#[cfg(feature = "asn1")]
impl TryFrom<(&Oid, Option<&Oid>)> for SignatureTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: (&Oid, Option<&Oid>)) -> Result<Self, Self::Error> {
    if value.0 == &OID_KEY_TYPE_EC_PUBLIC_KEY
      && let Some(elem) = value.1
    {
      if elem == &OID_EC_P256 {
        return Ok(Self::P256);
      } else if elem == &OID_NIST_EC_P384 {
        return Ok(Self::P384);
      }
    } else if value.0 == &OID_SIG_ED25519 {
      return Ok(Self::Ed25519);
    }
    Err(CryptoError::UnsupportedSignatureOid.into())
  }
}
