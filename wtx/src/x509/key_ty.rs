use crate::{
  asn1::{
    OID_EC_P256, OID_KEY_TYPE_EC_PUBLIC_KEY, OID_NIST_EC_P384, OID_NIST_HASH_SHA256,
    OID_NIST_HASH_SHA384, OID_PKCS1_RSAENCRYPTION, OID_PKCS1_RSASSAPSS, OID_SIG_ED25519, Oid,
  },
  crypto::CryptoError,
  misc::Lease,
  x509::{AlgorithmIdentifier, Certificate, Pkcs8, SubjectPublicKeyInfo},
};

/// Key Type
///
/// Can identify a Public Key algorithm or a Secret Key algorithm. However, this structure does
/// not identify signature algorithms.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KeyTy {
  /// Ecdsa P256
  #[default]
  EcdsaP256,
  /// Ecdsa P384
  EcdsaP384,
  /// Ed25519
  Ed25519,
  /// RSA PKCS1
  RsaPkcs1,
  /// RSA PSS SHA256
  RsaPssSha256,
  /// RSA PSS SHA384
  RsaPssSha384,
}

impl TryFrom<(&Oid, &Option<Oid>)> for KeyTy {
  type Error = crate::Error;

  #[inline]
  fn try_from((alg_oid, params_oid): (&Oid, &Option<Oid>)) -> Result<Self, Self::Error> {
    Ok(match alg_oid {
      oid if oid == &OID_PKCS1_RSAENCRYPTION => Self::RsaPkcs1,
      oid if oid == &OID_PKCS1_RSASSAPSS => match params_oid {
        Some(el) if el == &OID_NIST_HASH_SHA256 => Self::RsaPssSha256,
        Some(el) if el == &OID_NIST_HASH_SHA384 => Self::RsaPssSha384,
        _ => return Err(CryptoError::UnsupportedKeyOid.into()),
      },
      oid if oid == &OID_SIG_ED25519 => Self::Ed25519,
      oid if oid == &OID_KEY_TYPE_EC_PUBLIC_KEY => match params_oid {
        Some(curve) if curve == &OID_EC_P256 => Self::EcdsaP256,
        Some(curve) if curve == &OID_NIST_EC_P384 => Self::EcdsaP384,
        _ => return Err(CryptoError::UnsupportedKeyOid.into()),
      },

      _ => return Err(CryptoError::UnsupportedKeyOid.into()),
    })
  }
}

impl<B> TryFrom<&AlgorithmIdentifier<B>> for KeyTy
where
  B: Lease<[u8]>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &AlgorithmIdentifier<B>) -> Result<Self, Self::Error> {
    KeyTy::try_from((&value.algorithm, &value.params_oid()))
  }
}

impl<B> TryFrom<&Pkcs8<B>> for KeyTy
where
  B: Lease<[u8]>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &Pkcs8<B>) -> Result<Self, Self::Error> {
    KeyTy::try_from(&value.private_key_algorithm)
  }
}

impl<B> TryFrom<&SubjectPublicKeyInfo<B>> for KeyTy
where
  B: Lease<[u8]>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &SubjectPublicKeyInfo<B>) -> Result<Self, Self::Error> {
    KeyTy::try_from(&value.algorithm)
  }
}

impl<B> TryFrom<&Certificate<B>> for KeyTy
where
  B: Lease<[u8]>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &Certificate<B>) -> Result<Self, Self::Error> {
    KeyTy::try_from(&value.tbs_certificate().subject_public_key_info)
  }
}
