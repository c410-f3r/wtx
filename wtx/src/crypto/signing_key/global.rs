use crate::{
  crypto::{HashTy, SigningKey, SigningOutput},
  rng::CryptoRng,
};

type EcdsaP256SigningKeyTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::EcdsaP256SigningKeyRing,
  feature = "crypto-graviola" => crate::crypto::EcdsaP256SigningKeyGraviola,
  feature = "crypto-alr" => crate::crypto::EcdsaP256SigningKeyAlr,
  feature = "crypto-ruco" => crate::crypto::EcdsaP256SigningKeyRuco,
  _ => crate::crypto::SigningKeyDummy::<[u8; 64]>,
};

type EcdsaP384SigningKeyTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::EcdsaP384SigningKeyRing,
  feature = "crypto-graviola" => crate::crypto::EcdsaP384SigningKeyGraviola,
  feature = "crypto-alr" => crate::crypto::EcdsaP384SigningKeyAlr,
  feature = "crypto-ruco" => crate::crypto::EcdsaP384SigningKeyRuco,
  _ => crate::crypto::SigningKeyDummy::<[u8; 96]>,
};

type Ed25519SigningKeyTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Ed25519SigningKeyRing,
  feature = "crypto-graviola" => crate::crypto::Ed25519SigningKeyGraviola,
  feature = "crypto-alr" => crate::crypto::Ed25519SigningKeyAlr,
  feature = "crypto-ruco" => crate::crypto::Ed25519SigningKeyRuco,
  _ => crate::crypto::SigningKeyDummy::<[u8; 64]>,
};

type RsaPkcs1SigningKeyTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::RsaPkcs1SigningKeyRing,
  feature = "crypto-graviola" => crate::crypto::RsaPkcs1SigningKeyGraviola,
  feature = "crypto-alr" => crate::crypto::RsaPkcs1SigningKeyAlr,
  feature = "crypto-ruco" => crate::crypto::RsaPkcs1SigningKeyRuco,
  _ => crate::crypto::SigningKeyDummy::<[u8; 0]>,
};

type RsaPssSigningKeyTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::RsaPssSigningKeyRing,
  feature = "crypto-graviola" => crate::crypto::RsaPssSigningKeyGraviola,
  feature = "crypto-alr" => crate::crypto::RsaPssSigningKeyAlr,
  feature = "crypto-ruco" => crate::crypto::RsaPssSigningKeyRuco,
  _ => crate::crypto::SigningKeyDummy::<[u8; 0]>,
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct EcdsaP256SigningKeyGlobal(EcdsaP256SigningKeyTy);

impl SigningKey for EcdsaP256SigningKeyGlobal {
  type Signature = <EcdsaP256SigningKeyTy as SigningKey>::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self(EcdsaP256SigningKeyTy::from_pkcs8(bytes, hash_ty)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    self.0.sign(msg, rng)
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    EcdsaP256SigningKeyTy::validate(msg, pk, so)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct EcdsaP384SigningKeyGlobal(EcdsaP384SigningKeyTy);

impl SigningKey for EcdsaP384SigningKeyGlobal {
  type Signature = <EcdsaP384SigningKeyTy as SigningKey>::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self(EcdsaP384SigningKeyTy::from_pkcs8(bytes, hash_ty)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    self.0.sign(msg, rng)
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    EcdsaP384SigningKeyTy::validate(msg, pk, so)
  }
}

/// A structure that delegates execution to the selected crypto backend for Ed25519.
#[derive(Debug)]
pub struct Ed25519SigningKeyGlobal(Ed25519SigningKeyTy);

impl SigningKey for Ed25519SigningKeyGlobal {
  type Signature = <Ed25519SigningKeyTy as SigningKey>::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self(Ed25519SigningKeyTy::from_pkcs8(bytes, hash_ty)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    self.0.sign(msg, rng)
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    Ed25519SigningKeyTy::validate(msg, pk, so)
  }
}

/// A structure that delegates execution to the selected crypto backend for RSA PKCS1.
#[derive(Debug)]
pub struct RsaPkcs1SigningKeyGlobal(RsaPkcs1SigningKeyTy);

impl SigningKey for RsaPkcs1SigningKeyGlobal {
  type Signature = <RsaPkcs1SigningKeyTy as SigningKey>::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self(RsaPkcs1SigningKeyTy::from_pkcs8(bytes, hash_ty)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    self.0.sign(msg, rng)
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    RsaPkcs1SigningKeyTy::validate(msg, pk, so)
  }
}

/// A structure that delegates execution to the selected crypto backend for RSA PSS.
#[derive(Debug)]
pub struct RsaPssSigningKeyGlobal(RsaPssSigningKeyTy);

impl SigningKey for RsaPssSigningKeyGlobal {
  type Signature = <RsaPssSigningKeyTy as SigningKey>::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self(RsaPssSigningKeyTy::from_pkcs8(bytes, hash_ty)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    self.0.sign(msg, rng)
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    RsaPssSigningKeyTy::validate(msg, pk, so)
  }
}
