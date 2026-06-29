use crate::crypto::{
  Ed25519Global, P256SignatureGlobal, P384SignatureGlobal, RsaPssRsaeSha256Global,
  RsaPssRsaeSha384Global, Signature as _,
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
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

create_enum! {
  /// Specifies the group or curve used for agreements.
  #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
  pub enum SignatureTy<u16> {
    /// ECDSA Secp256r1 SHA-256
    EcdsaSecp256r1Sha256 = (1027, "EcdsaSecp256r1Sha256"),
    /// ECDSA Secp384r1 SHA-384
    EcdsaSecp384r1Sha384 = (1283, "EcdsaSecp384r1Sha384"),
    /// Ed25519
    Ed25519 = (2055, "Ed25519"),
    /// RSA PSS RSAE SHA-256
    RsaPssRsaeSha256 = (2052, "RsaPssRsaeSha256"),
    /// RSA PSS RSAE SHA-384
    RsaPssRsaeSha384 = (2053, "RsaPssRsaeSha384"),
  }
}

impl SignatureTy {
  /// If the current instance has any RSA type.
  #[inline]
  pub const fn is_rsa(self) -> bool {
    matches!(self, Self::RsaPssRsaeSha256 | Self::RsaPssRsaeSha384)
  }

  /// Calls the validation method that corresponds to the current instance variant and the selected
  /// crypto backend.
  #[inline]
  pub fn validate_signature(&self, pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    match self {
      SignatureTy::EcdsaSecp256r1Sha256 => P256SignatureGlobal::validate(pk, msg, signature),
      SignatureTy::EcdsaSecp384r1Sha384 => P384SignatureGlobal::validate(pk, msg, signature),
      SignatureTy::Ed25519 => Ed25519Global::validate(pk, msg, signature),
      SignatureTy::RsaPssRsaeSha256 => RsaPssRsaeSha256Global::validate(pk, msg, signature),
      SignatureTy::RsaPssRsaeSha384 => RsaPssRsaeSha384Global::validate(pk, msg, signature),
    }
  }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SignatureTy {
  #[inline]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let str: &str = Deserialize::deserialize(deserializer)?;
    Self::try_from(str.as_bytes()).map_err(serde::de::Error::custom)
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

impl From<SignatureTy> for &'static str {
  #[inline]
  fn from(from: SignatureTy) -> Self {
    match from {
      SignatureTy::EcdsaSecp256r1Sha256 => "EcdsaSecp256r1Sha256",
      SignatureTy::EcdsaSecp384r1Sha384 => "EcdsaSecp384r1Sha384",
      SignatureTy::Ed25519 => "Ed25519",
      SignatureTy::RsaPssRsaeSha256 => "RsaPssRsaeSha256",
      SignatureTy::RsaPssRsaeSha384 => "RsaPssRsaeSha384",
    }
  }
}

#[cfg(feature = "asn1")]
impl TryFrom<(&Oid, Option<&Oid>)> for SignatureTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: (&Oid, Option<&Oid>)) -> Result<Self, Self::Error> {
    let (sig_alg, param) = value;
    match sig_alg {
      oid if oid == &OID_SIG_ED25519 => return Ok(Self::Ed25519),

      oid if oid == &OID_SIG_ECDSA_WITH_SHA256 => return Ok(Self::EcdsaSecp256r1Sha256),
      oid if oid == &OID_SIG_ECDSA_WITH_SHA384 => return Ok(Self::EcdsaSecp384r1Sha384),
      oid if oid == &OID_KEY_TYPE_EC_PUBLIC_KEY => match param {
        Some(el) if el == &OID_EC_P256 => return Ok(Self::EcdsaSecp256r1Sha256),
        Some(el) if el == &OID_NIST_EC_P384 => return Ok(Self::EcdsaSecp384r1Sha384),
        _ => {}
      },

      oid if oid == &OID_PKCS1_RSASSAPSS => match param {
        Some(el) if el == &OID_NIST_HASH_SHA256 => return Ok(Self::RsaPssRsaeSha256),
        Some(el) if el == &OID_NIST_HASH_SHA384 => return Ok(Self::RsaPssRsaeSha384),
        _ => {}
      },
      _ => {}
    }
    Err(CryptoError::UnsupportedSignatureOid.into())
  }
}
