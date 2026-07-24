use crate::{
  crypto::{
    EcdsaP256SigningKeyRuco, EcdsaP384SigningKeyRuco, Ed25519SigningKeyRuco, HashTy,
    RsaPkcs1SigningKeyRuco, RsaPssSigningKeyRuco, SigningOutput, signing_key::SigningKey,
  },
  rng::CryptoRng,
};
use alloc::boxed::Box;
use pkcs8::DecodePrivateKey as _;
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey as _, pkcs1v15, pss};
use signature::{RandomizedSigner as _, Signer as _, Verifier as _};
use spki::DecodePublicKey as _;

impl SigningKey for EcdsaP256SigningKeyRuco {
  type Signature = [u8; 64];

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(p256::ecdsa::SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let signature: p256::ecdsa::Signature = self.0.try_sign_with_rng(rng, msg)?;
    Ok(SigningOutput::new(HashTy::Sha256, signature.to_bytes().into()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let sig = if so.signature().len() == 64 {
      p256::ecdsa::Signature::from_slice(so.signature())?
    } else {
      p256::ecdsa::Signature::from_der(so.signature())?
    };
    p256::ecdsa::VerifyingKey::from_sec1_bytes(pk)?.verify(msg, &sig)?;
    Ok(())
  }
}

impl SigningKey for EcdsaP384SigningKeyRuco {
  type Signature = [u8; 96];

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(p384::ecdsa::SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let signature: p384::ecdsa::Signature = self.0.try_sign_with_rng(rng, msg)?;
    Ok(SigningOutput::new(HashTy::Sha384, signature.to_bytes().into()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let sig = if so.signature().len() == 96 {
      p384::ecdsa::Signature::from_slice(so.signature())?
    } else {
      p384::ecdsa::Signature::from_der(so.signature())?
    };
    p384::ecdsa::VerifyingKey::from_sec1_bytes(pk)?.verify(msg, &sig)?;
    Ok(())
  }
}

impl SigningKey for Ed25519SigningKeyRuco {
  type Signature = [u8; 64];

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(ed25519_dalek::SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    Ok(SigningOutput::new(HashTy::Sha512, self.0.try_sign(msg)?.to_bytes()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    ed25519_dalek::VerifyingKey::from_bytes(pk.try_into()?)?
      .verify(msg, &ed25519_dalek::Signature::from_slice(so.signature())?)?;
    Ok(())
  }
}

impl SigningKey for RsaPkcs1SigningKeyRuco {
  type Signature = Box<[u8]>;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(match hash_ty {
      HashTy::Sha256 => RsaPkcs1SigningKeyRuco::Sha256(pkcs1v15::SigningKey::new(
        rsa::RsaPrivateKey::from_pkcs8_der(bytes)?,
      )),
      HashTy::Sha384 => RsaPkcs1SigningKeyRuco::Sha384(pkcs1v15::SigningKey::new(
        rsa::RsaPrivateKey::from_pkcs8_der(bytes)?,
      )),
      HashTy::Sha512 => RsaPkcs1SigningKeyRuco::Sha512(pkcs1v15::SigningKey::new(
        rsa::RsaPrivateKey::from_pkcs8_der(bytes)?,
      )),
    })
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let (hash_ty, signature) = match self {
      RsaPkcs1SigningKeyRuco::Sha256(el) => (HashTy::Sha256, el.try_sign_with_rng(rng, msg)?),
      RsaPkcs1SigningKeyRuco::Sha384(el) => (HashTy::Sha384, el.try_sign_with_rng(rng, msg)?),
      RsaPkcs1SigningKeyRuco::Sha512(el) => (HashTy::Sha512, el.try_sign_with_rng(rng, msg)?),
    };
    Ok(SigningOutput::new(hash_ty, signature.into()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let rpk =
      RsaPublicKey::from_public_key_der(pk).or_else(|_| RsaPublicKey::from_pkcs1_der(pk))?;
    let sig = pkcs1v15::Signature::try_from(&**so.signature())?;
    match so.hash_ty() {
      HashTy::Sha256 => pkcs1v15::VerifyingKey::<sha2::Sha256>::new(rpk).verify(msg, &sig)?,
      HashTy::Sha384 => pkcs1v15::VerifyingKey::<sha2::Sha384>::new(rpk).verify(msg, &sig)?,
      HashTy::Sha512 => pkcs1v15::VerifyingKey::<sha2::Sha512>::new(rpk).verify(msg, &sig)?,
    }
    Ok(())
  }
}

impl SigningKey for RsaPssSigningKeyRuco {
  type Signature = Box<[u8]>;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(match hash_ty {
      HashTy::Sha256 => RsaPssSigningKeyRuco::Sha256(pss::SigningKey::new(
        rsa::RsaPrivateKey::from_pkcs8_der(bytes)?,
      )),
      HashTy::Sha384 => RsaPssSigningKeyRuco::Sha384(pss::SigningKey::new(
        rsa::RsaPrivateKey::from_pkcs8_der(bytes)?,
      )),
      HashTy::Sha512 => RsaPssSigningKeyRuco::Sha512(pss::SigningKey::new(
        rsa::RsaPrivateKey::from_pkcs8_der(bytes)?,
      )),
    })
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let (hash_ty, signature) = match self {
      RsaPssSigningKeyRuco::Sha256(el) => (HashTy::Sha256, el.try_sign_with_rng(rng, msg)?),
      RsaPssSigningKeyRuco::Sha384(el) => (HashTy::Sha384, el.try_sign_with_rng(rng, msg)?),
      RsaPssSigningKeyRuco::Sha512(el) => (HashTy::Sha512, el.try_sign_with_rng(rng, msg)?),
    };
    Ok(SigningOutput::new(hash_ty, signature.into()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let rpk =
      RsaPublicKey::from_public_key_der(pk).or_else(|_| RsaPublicKey::from_pkcs1_der(pk))?;
    let sig = pss::Signature::try_from(&**so.signature())?;
    match so.hash_ty() {
      HashTy::Sha256 => pss::VerifyingKey::<sha2::Sha256>::new(rpk).verify(msg, &sig)?,
      HashTy::Sha384 => pss::VerifyingKey::<sha2::Sha384>::new(rpk).verify(msg, &sig)?,
      HashTy::Sha512 => pss::VerifyingKey::<sha2::Sha512>::new(rpk).verify(msg, &sig)?,
    }
    Ok(())
  }
}
