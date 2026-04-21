use crate::crypto::{
  GlobalEd25519, GlobalP256Signature, GlobalP384Signature, GlobalRsaPssRsaeSha256,
  GlobalRsaPssRsaeSha384, Signature,
};
#[cfg(feature = "asn1")]
use crate::{
  asn1::{
    OID_EC_P256, OID_KEY_TYPE_EC_PUBLIC_KEY, OID_NIST_EC_P384, OID_NIST_HASH_SHA256,
    OID_NIST_HASH_SHA384, OID_PKCS1_RSASSAPSS, OID_SIG_ECDSA_WITH_SHA256,
    OID_SIG_ECDSA_WITH_SHA384, OID_SIG_ED25519, Oid,
  },
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
    Ed25519 = (1, "ED25519"),
    /// RSA PSS RSAE SHA-256
    RsaPssRsaeSha256 = (2, "RSAPSSRSAESHA256"),
    /// RSA PSS RSAE SHA-384
    RsaPssRsaeSha384 = (3, "RSAPSSRSAESHA384"),
    /// Secp256r1
    Secp256r1 = (4, "SECP256R1"),
    /// Secp384r1
    Secp384r1 = (5, "SECP384R1"),
  }
}

impl SignatureTy {
  /// Calls [`Signature::validate`] according to the current instance value and the selected crypto
  /// backend.
  #[inline]
  pub fn validate_signature(&self, pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    match self {
      SignatureTy::Ed25519 => GlobalEd25519::validate(pk, msg, signature),
      SignatureTy::RsaPssRsaeSha256 => GlobalRsaPssRsaeSha256::validate(pk, msg, signature),
      SignatureTy::RsaPssRsaeSha384 => GlobalRsaPssRsaeSha384::validate(pk, msg, signature),
      SignatureTy::Secp256r1 => GlobalP256Signature::validate(pk, msg, signature),
      SignatureTy::Secp384r1 => GlobalP384Signature::validate(pk, msg, signature),
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
      SignatureTy::RsaPssRsaeSha256 => "RSAPSSRSAESHA256",
      SignatureTy::RsaPssRsaeSha384 => "RSAPSSRSAESHA384",
      SignatureTy::Secp256r1 => "SECP256R1",
      SignatureTy::Secp384r1 => "SECP384R1",
    }
  }
}

#[cfg(feature = "asn1")]
impl TryFrom<(&Oid, Option<&Oid>)> for SignatureTy {
  type Error = crate::Error;

  fn try_from(value: (&Oid, Option<&Oid>)) -> Result<Self, Self::Error> {
    let (sig_alg, param) = value;
    match sig_alg {
      oid if oid == &OID_SIG_ED25519 => return Ok(Self::Ed25519),

      oid if oid == &OID_SIG_ECDSA_WITH_SHA256 => return Ok(Self::Secp256r1),
      oid if oid == &OID_SIG_ECDSA_WITH_SHA384 => return Ok(Self::Secp384r1),
      oid if oid == &OID_KEY_TYPE_EC_PUBLIC_KEY => match param {
        Some(param) if param == &OID_EC_P256 => return Ok(Self::Secp256r1),
        Some(param) if param == &OID_NIST_EC_P384 => return Ok(Self::Secp384r1),
        _ => {}
      },

      oid if oid == &OID_PKCS1_RSASSAPSS => match param {
        Some(param) if param == &OID_NIST_HASH_SHA256 => return Ok(Self::RsaPssRsaeSha256),
        Some(param) if param == &OID_NIST_HASH_SHA384 => return Ok(Self::RsaPssRsaeSha384),
        _ => {}
      },
      _ => {}
    }
    Err(CryptoError::UnsupportedSignatureOid.into())
  }
}
