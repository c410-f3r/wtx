use crate::{
  crypto::{
    DynSigningOutput, EcdsaP256SigningKeyGlobal, EcdsaP384SigningKeyGlobal,
    Ed25519SigningKeyGlobal, RsaPkcs1SigningKeyGlobal, RsaPssSigningKeyGlobal, SigningKey as _,
  },
  rng::CryptoRng,
};

/// Specifies the algorithm used for certificates.
#[derive(Debug)]
pub enum DynSigningKey {
  /// ECDSA P256
  EcdsaP256(EcdsaP256SigningKeyGlobal),
  /// ECDSA P384
  EcdsaP384(EcdsaP384SigningKeyGlobal),
  /// Ed25519
  Ed25519(Ed25519SigningKeyGlobal),
  /// RSA PKCS1
  RsaPkcs1(RsaPkcs1SigningKeyGlobal),
  /// RSA PSS
  RsaPss(RsaPssSigningKeyGlobal),
}

impl DynSigningKey {
  /// Calls the signing method that corresponds to the current instance variant and the selected
  /// crypto backend.
  #[inline]
  pub fn sign<RNG>(&mut self, msg: &[u8], rng: &mut RNG) -> crate::Result<DynSigningOutput>
  where
    RNG: CryptoRng,
  {
    Ok(match self {
      Self::EcdsaP256(el) => {
        DynSigningOutput::EcdsaP256(EcdsaP256SigningKeyGlobal::sign(el, msg, rng)?)
      }
      Self::EcdsaP384(el) => {
        DynSigningOutput::EcdsaP384(EcdsaP384SigningKeyGlobal::sign(el, msg, rng)?)
      }
      Self::Ed25519(el) => DynSigningOutput::Ed25519(Ed25519SigningKeyGlobal::sign(el, msg, rng)?),
      Self::RsaPkcs1(el) => {
        DynSigningOutput::RsaPkcs1(RsaPkcs1SigningKeyGlobal::sign(el, msg, rng)?)
      }
      Self::RsaPss(el) => DynSigningOutput::RsaPss(RsaPssSigningKeyGlobal::sign(el, msg, rng)?),
    })
  }
}
