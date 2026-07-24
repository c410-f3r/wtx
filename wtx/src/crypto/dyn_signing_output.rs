use crate::crypto::{
  EcdsaP256SigningKeyGlobal, EcdsaP384SigningKeyGlobal, Ed25519SigningKeyGlobal,
  RsaPkcs1SigningKeyGlobal, RsaPssSigningKeyGlobal, SigningKey, SigningOutput,
};
use core::fmt::{Debug, Formatter};

/// Specifies the algorithm used for certificates.
pub enum DynSigningOutput {
  /// ECDSA P256
  EcdsaP256(SigningOutput<<EcdsaP256SigningKeyGlobal as SigningKey>::Signature>),
  /// ECDSA P384
  EcdsaP384(SigningOutput<<EcdsaP384SigningKeyGlobal as SigningKey>::Signature>),
  /// Ed25519
  Ed25519(SigningOutput<<Ed25519SigningKeyGlobal as SigningKey>::Signature>),
  /// RSA Pkcs1
  RsaPkcs1(SigningOutput<<RsaPkcs1SigningKeyGlobal as SigningKey>::Signature>),
  /// RSA PSS
  RsaPss(SigningOutput<<RsaPssSigningKeyGlobal as SigningKey>::Signature>),
}

#[allow(clippy::match_same_arms, reason = "depends on feature")]
impl AsRef<[u8]> for DynSigningOutput {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    match self {
      DynSigningOutput::EcdsaP256(el) => el.signature().as_ref(),
      DynSigningOutput::EcdsaP384(el) => el.signature().as_ref(),
      DynSigningOutput::Ed25519(el) => el.signature().as_ref(),
      DynSigningOutput::RsaPkcs1(el) => el.signature().as_ref(),
      DynSigningOutput::RsaPss(el) => el.signature().as_ref(),
    }
  }
}

impl Debug for DynSigningOutput {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SignatureSignOutput").finish()
  }
}
