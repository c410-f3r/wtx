#[cfg(feature = "asn1")]
use crate::{
  asn1::{
    OID_EC_P256, OID_KEY_TYPE_EC_PUBLIC_KEY, OID_NIST_EC_P384, OID_NIST_HASH_SHA256,
    OID_NIST_HASH_SHA384, OID_PKCS1_RSASSAPSS, OID_SIG_ECDSA_WITH_SHA256,
    OID_SIG_ECDSA_WITH_SHA384, OID_SIG_ED25519, Oid,
  },
  crypto::CryptoError,
};
use crate::{
  crypto::{
    Ed25519Global, P256SignatureGlobal, P384SignatureGlobal, RsaPssRsaeSha256Global,
    RsaPssRsaeSha384Global, SignKey as _, Signature,
  },
  rng::CryptoRng,
};
use core::fmt::{Debug, Formatter};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

create_enum! {
  /// Specifies the algorithm used for certificates
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

  /// Creates the signing structure that corresponds to the current instance variant and the
  /// selected crypto backend.
  #[inline]
  pub fn sign_key_from_pkcs8(self, bytes: &[u8]) -> crate::Result<SignatureSignKey> {
    Ok(match self {
      Self::EcdsaSecp256r1Sha256 => SignatureSignKey::EcdsaSecp256r1Sha256(<_>::from_pkcs8(bytes)?),
      Self::EcdsaSecp384r1Sha384 => SignatureSignKey::EcdsaSecp384r1Sha384(<_>::from_pkcs8(bytes)?),
      Self::Ed25519 => SignatureSignKey::Ed25519(<_>::from_pkcs8(bytes)?),
      Self::RsaPssRsaeSha256 => SignatureSignKey::RsaPssRsaeSha256(<_>::from_pkcs8(bytes)?),
      Self::RsaPssRsaeSha384 => SignatureSignKey::RsaPssRsaeSha384(<_>::from_pkcs8(bytes)?),
    })
  }

  /// Calls the validation method that corresponds to the current instance variant and the selected
  /// crypto backend.
  #[inline]
  pub fn validate_signature(self, pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    match self {
      Self::EcdsaSecp256r1Sha256 => P256SignatureGlobal::validate(pk, msg, signature),
      Self::EcdsaSecp384r1Sha384 => P384SignatureGlobal::validate(pk, msg, signature),
      Self::Ed25519 => Ed25519Global::validate(pk, msg, signature),
      Self::RsaPssRsaeSha256 => RsaPssRsaeSha256Global::validate(pk, msg, signature),
      Self::RsaPssRsaeSha384 => RsaPssRsaeSha384Global::validate(pk, msg, signature),
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

/// Specifies the algorithm used for certificates.
#[derive(Debug)]
pub enum SignatureSignKey {
  /// ECDSA Secp256r1 SHA-256
  EcdsaSecp256r1Sha256(<P256SignatureGlobal as Signature>::SignKey),
  /// ECDSA Secp384r1 SHA-384
  EcdsaSecp384r1Sha384(<P384SignatureGlobal as Signature>::SignKey),
  /// Ed25519
  Ed25519(<Ed25519Global as Signature>::SignKey),
  /// RSA PSS RSAE SHA-256
  RsaPssRsaeSha256(<RsaPssRsaeSha256Global as Signature>::SignKey),
  /// RSA PSS RSAE SHA-384
  RsaPssRsaeSha384(<RsaPssRsaeSha384Global as Signature>::SignKey),
}

impl SignatureSignKey {
  /// Calls the signing method that corresponds to the current instance variant and the selected
  /// crypto backend.
  #[inline]
  pub fn sign<RNG>(&mut self, rng: &mut RNG, msg: &[u8]) -> crate::Result<SignatureSignOutput>
  where
    RNG: CryptoRng,
  {
    Ok(match self {
      Self::EcdsaSecp256r1Sha256(el) => {
        SignatureSignOutput::EcdsaSecp256r1Sha256(P256SignatureGlobal::sign(rng, el, msg)?)
      }
      Self::EcdsaSecp384r1Sha384(el) => {
        SignatureSignOutput::EcdsaSecp384r1Sha384(P384SignatureGlobal::sign(rng, el, msg)?)
      }
      Self::Ed25519(el) => SignatureSignOutput::Ed25519(Ed25519Global::sign(rng, el, msg)?),
      Self::RsaPssRsaeSha256(el) => {
        SignatureSignOutput::RsaPssRsaeSha256(RsaPssRsaeSha256Global::sign(rng, el, msg)?)
      }
      Self::RsaPssRsaeSha384(el) => {
        SignatureSignOutput::RsaPssRsaeSha384(RsaPssRsaeSha384Global::sign(rng, el, msg)?)
      }
    })
  }
}

/// Specifies the algorithm used for certificates.
pub enum SignatureSignOutput {
  /// ECDSA Secp256r1 SHA-256
  EcdsaSecp256r1Sha256(<P256SignatureGlobal as Signature>::SignOutput),
  /// ECDSA Secp384r1 SHA-384
  EcdsaSecp384r1Sha384(<P384SignatureGlobal as Signature>::SignOutput),
  /// Ed25519
  Ed25519(<Ed25519Global as Signature>::SignOutput),
  /// RSA PSS RSAE SHA-256
  RsaPssRsaeSha256(<RsaPssRsaeSha256Global as Signature>::SignOutput),
  /// RSA PSS RSAE SHA-384
  RsaPssRsaeSha384(<RsaPssRsaeSha384Global as Signature>::SignOutput),
}

#[allow(clippy::match_same_arms, reason = "depends on feature")]
impl AsRef<[u8]> for SignatureSignOutput {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    match self {
      SignatureSignOutput::EcdsaSecp256r1Sha256(el) => el.as_ref(),
      SignatureSignOutput::EcdsaSecp384r1Sha384(el) => el.as_ref(),
      SignatureSignOutput::Ed25519(el) => el.as_ref(),
      SignatureSignOutput::RsaPssRsaeSha256(el) => el.as_ref(),
      SignatureSignOutput::RsaPssRsaeSha384(el) => el.as_ref(),
    }
  }
}

impl Debug for SignatureSignOutput {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SignatureSignOutput").finish()
  }
}
