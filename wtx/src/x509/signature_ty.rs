use crate::{
  asn1::{
    OID_NIST_HASH_SHA256, OID_NIST_HASH_SHA384, OID_PKCS1_RSASSAPSS, OID_PKCS1_SHA256WITHRSA,
    OID_PKCS1_SHA384WITHRSA, OID_SIG_ECDSA_WITH_SHA256, OID_SIG_ECDSA_WITH_SHA384, OID_SIG_ED25519,
    Oid,
  },
  crypto::{
    CryptoError, DynSigningKey, EcdsaP256SigningKeyGlobal, EcdsaP384SigningKeyGlobal,
    Ed25519SigningKeyGlobal, HashTy, RsaPkcs1SigningKeyGlobal, RsaPssSigningKeyGlobal,
    SigningKey as _, SigningOutput,
  },
  misc::Lease,
  x509::{AlgorithmIdentifier, Certificate},
};
use core::fmt::Debug;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Specifies the algorithm used for signing a certificate.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SignatureTy {
  /// ECDSA Secp256r1
  #[default]
  EcdsaP256,
  /// ECDSA Secp384r1
  EcdsaP384,
  /// Ed25519
  Ed25519,
  /// RSA PKCS1 SHA256
  RsaPkcs1Sha256,
  /// RSA PKCS1 SHA384
  RsaPkcs1Sha384,
  /// RSA PSS SHA256
  RsaPssSha256,
  /// RSA PSS SHA384
  RsaPssSha384,
}

impl SignatureTy {
  /// See [`HashTy`].
  #[inline]
  pub const fn hash_ty(&self) -> HashTy {
    match self {
      SignatureTy::EcdsaP256 | SignatureTy::RsaPkcs1Sha256 | SignatureTy::RsaPssSha256 => {
        HashTy::Sha256
      }
      SignatureTy::EcdsaP384 | SignatureTy::RsaPkcs1Sha384 | SignatureTy::RsaPssSha384 => {
        HashTy::Sha384
      }
      SignatureTy::Ed25519 => HashTy::Sha512,
    }
  }

  /// Number of variants
  #[inline]
  pub const fn len() -> usize {
    7
  }

  /// Creates the signing structure that corresponds to the current instance variant and the
  /// selected crypto backend.
  #[inline]
  pub fn sign_key_from_pkcs8(self, bytes: &[u8]) -> crate::Result<DynSigningKey> {
    Ok(match self {
      Self::EcdsaP256 => DynSigningKey::EcdsaP256(<_>::from_pkcs8(bytes, HashTy::Sha256)?),
      Self::EcdsaP384 => DynSigningKey::EcdsaP384(<_>::from_pkcs8(bytes, HashTy::Sha384)?),
      Self::Ed25519 => DynSigningKey::Ed25519(<_>::from_pkcs8(bytes, HashTy::Sha512)?),
      Self::RsaPkcs1Sha256 => DynSigningKey::RsaPkcs1(<_>::from_pkcs8(bytes, HashTy::Sha256)?),
      Self::RsaPkcs1Sha384 => DynSigningKey::RsaPkcs1(<_>::from_pkcs8(bytes, HashTy::Sha384)?),
      Self::RsaPssSha256 => DynSigningKey::RsaPss(<_>::from_pkcs8(bytes, HashTy::Sha256)?),
      Self::RsaPssSha384 => DynSigningKey::RsaPss(<_>::from_pkcs8(bytes, HashTy::Sha384)?),
    })
  }

  /// Calls the validation method that corresponds to the current instance variant and the selected
  /// crypto backend.
  #[inline]
  pub fn validate_signature(self, msg: &[u8], pk: &[u8], signature: &[u8]) -> crate::Result<()> {
    match self {
      SignatureTy::EcdsaP256 => {
        EcdsaP256SigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha256, signature))
      }
      SignatureTy::EcdsaP384 => {
        EcdsaP384SigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha384, signature))
      }
      SignatureTy::Ed25519 => {
        Ed25519SigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha512, signature))
      }
      SignatureTy::RsaPkcs1Sha256 => {
        RsaPkcs1SigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha256, signature))
      }
      SignatureTy::RsaPkcs1Sha384 => {
        RsaPkcs1SigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha384, signature))
      }
      SignatureTy::RsaPssSha256 => {
        RsaPssSigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha256, signature))
      }
      SignatureTy::RsaPssSha384 => {
        RsaPssSigningKeyGlobal::validate(msg, pk, &SigningOutput::new(HashTy::Sha384, signature))
      }
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
    Self::try_from(str).map_err(serde::de::Error::custom)
  }
}

#[cfg(feature = "serde")]
impl Serialize for SignatureTy {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str((*self).into())
  }
}

impl From<SignatureTy> for &'static str {
  #[inline]
  fn from(from: SignatureTy) -> Self {
    match from {
      SignatureTy::EcdsaP256 => "EcdsaP256",
      SignatureTy::EcdsaP384 => "EcdsaP384",
      SignatureTy::Ed25519 => "Ed25519",
      SignatureTy::RsaPkcs1Sha256 => "RsaPkcs1Sha256",
      SignatureTy::RsaPkcs1Sha384 => "RsaPkcs1Sha384",
      SignatureTy::RsaPssSha256 => "RsaPssSha256",
      SignatureTy::RsaPssSha384 => "RsaPssSha384",
    }
  }
}

impl TryFrom<&str> for SignatureTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &str) -> crate::Result<Self> {
    Ok(match from {
      "EcdsaP256" => SignatureTy::EcdsaP256,
      "EcdsaP384" => SignatureTy::EcdsaP384,
      "Ed25519" => SignatureTy::Ed25519,
      "RsaPkcs1Sha256" => SignatureTy::RsaPkcs1Sha256,
      "RsaPkcs1Sha384" => SignatureTy::RsaPkcs1Sha384,
      "RsaPssSha256" => SignatureTy::RsaPssSha256,
      "RsaPssSha384" => SignatureTy::RsaPssSha384,
      _ => return Err(CryptoError::UnknownSignatureTy.into()),
    })
  }
}

impl TryFrom<&Oid> for SignatureTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(sig_alg: &Oid) -> Result<Self, Self::Error> {
    Ok(match sig_alg {
      oid if oid == &OID_SIG_ED25519 => Self::Ed25519,
      oid if oid == &OID_SIG_ECDSA_WITH_SHA256 => Self::EcdsaP256,
      oid if oid == &OID_SIG_ECDSA_WITH_SHA384 => Self::EcdsaP384,
      oid if oid == &OID_PKCS1_SHA256WITHRSA => Self::RsaPkcs1Sha256,
      oid if oid == &OID_PKCS1_SHA384WITHRSA => Self::RsaPkcs1Sha384,
      oid if oid == &OID_NIST_HASH_SHA256 => Self::RsaPssSha256,
      oid if oid == &OID_NIST_HASH_SHA384 => Self::RsaPssSha384,
      _ => return Err(CryptoError::UnsupportedSignatureOid.into()),
    })
  }
}

impl TryFrom<(&Oid, &Option<Oid>)> for SignatureTy {
  type Error = crate::Error;

  #[inline]
  fn try_from((sig_alg, sig_params): (&Oid, &Option<Oid>)) -> Result<Self, Self::Error> {
    Ok(match sig_alg {
      oid if oid == &OID_SIG_ED25519 => Self::Ed25519,
      oid if oid == &OID_SIG_ECDSA_WITH_SHA256 => Self::EcdsaP256,
      oid if oid == &OID_SIG_ECDSA_WITH_SHA384 => Self::EcdsaP384,
      oid if oid == &OID_PKCS1_SHA256WITHRSA => Self::RsaPkcs1Sha256,
      oid if oid == &OID_PKCS1_SHA384WITHRSA => Self::RsaPkcs1Sha384,
      oid if oid == &OID_PKCS1_RSASSAPSS => match sig_params {
        Some(el) if el == &OID_NIST_HASH_SHA256 => Self::RsaPssSha256,
        Some(el) if el == &OID_NIST_HASH_SHA384 => Self::RsaPssSha384,
        _ => return Err(CryptoError::UnsupportedSignatureOid.into()),
      },
      _ => return Err(CryptoError::UnsupportedSignatureOid.into()),
    })
  }
}

impl<B> TryFrom<&AlgorithmIdentifier<B>> for SignatureTy
where
  B: Lease<[u8]>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &AlgorithmIdentifier<B>) -> Result<Self, Self::Error> {
    SignatureTy::try_from((&value.algorithm, &value.params_oid()))
  }
}

impl<B> TryFrom<&Certificate<B>> for SignatureTy
where
  B: Lease<[u8]>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &Certificate<B>) -> Result<Self, Self::Error> {
    SignatureTy::try_from(value.signature_algorithm())
  }
}
