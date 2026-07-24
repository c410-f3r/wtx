use crate::{
  collections::Vector,
  crypto::{
    EcdsaP256SigningKeyGraviola, EcdsaP384SigningKeyGraviola, Ed25519SigningKeyGraviola, HashTy,
    RsaPkcs1SigningKeyGraviola, RsaPssSigningKeyGraviola, SigningOutput, signing_key::SigningKey,
  },
  rng::CryptoRng,
};
use alloc::boxed::Box;
use graviola::{
  hashing::{Sha256, Sha384},
  signing::{
    ecdsa::{self, P256, P384},
    eddsa::{Ed25519SigningKey, Ed25519VerifyingKey},
    rsa,
  },
};

impl SigningKey for EcdsaP256SigningKeyGraviola {
  type Signature = [u8; 64];

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(ecdsa::SigningKey::<P256>::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let mut signature = [0; 64];
    let _ = self.0.sign::<Sha256>(&[msg], &mut signature)?;
    Ok(SigningOutput::new(HashTy::Sha256, signature))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let vk = ecdsa::VerifyingKey::<P256>::from_x962_uncompressed(pk)?;
    if so.signature().len() == 64 {
      vk.verify::<Sha256>(&[msg], so.signature())?;
    } else {
      vk.verify_asn1::<Sha256>(&[msg], so.signature())?;
    }
    Ok(())
  }
}

impl SigningKey for EcdsaP384SigningKeyGraviola {
  type Signature = [u8; 96];

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(ecdsa::SigningKey::<P384>::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let mut signature = [0; 96];
    let _ = self.0.sign::<Sha384>(&[msg], &mut signature)?;
    Ok(SigningOutput::new(HashTy::Sha384, signature))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let vk = ecdsa::VerifyingKey::<P384>::from_x962_uncompressed(pk)?;
    if so.signature().len() == 96 {
      vk.verify::<Sha384>(&[msg], so.signature())?;
    } else {
      vk.verify_asn1::<Sha384>(&[msg], so.signature())?;
    }
    Ok(())
  }
}

impl SigningKey for Ed25519SigningKeyGraviola {
  type Signature = [u8; 64];

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(Ed25519SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    Ok(SigningOutput::new(HashTy::Sha512, self.0.sign(msg)))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    Ed25519VerifyingKey::from_bytes(pk)?.verify(so.signature(), msg)?;
    Ok(())
  }
}

impl SigningKey for RsaPkcs1SigningKeyGraviola {
  type Signature = Box<[u8]>;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self((hash_ty, rsa::SigningKey::from_pkcs8_der(bytes)?)))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let mut signature = Vector::from_vec(alloc::vec![0; self.0.1.modulus_len_bytes()]);
    let _ = match self.0.0 {
      HashTy::Sha256 => self.0.1.sign_pkcs1_sha256(&mut signature, msg)?,
      HashTy::Sha384 => self.0.1.sign_pkcs1_sha384(&mut signature, msg)?,
      HashTy::Sha512 => self.0.1.sign_pkcs1_sha512(&mut signature, msg)?,
    };
    Ok(SigningOutput::new(self.0.0, signature.into_vec().into_boxed_slice()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let vk = rsa::VerifyingKey::from_pkcs1_der(pk)?;
    match so.hash_ty() {
      HashTy::Sha256 => vk.verify_pkcs1_sha256(so.signature(), msg)?,
      HashTy::Sha384 => vk.verify_pkcs1_sha384(so.signature(), msg)?,
      HashTy::Sha512 => vk.verify_pkcs1_sha512(so.signature(), msg)?,
    }
    Ok(())
  }
}

impl SigningKey for RsaPssSigningKeyGraviola {
  type Signature = Box<[u8]>;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self((hash_ty, rsa::SigningKey::from_pkcs8_der(bytes)?)))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let mut signature = Vector::from_vec(alloc::vec![0; self.0.1.modulus_len_bytes()]);
    let _ = match self.0.0 {
      HashTy::Sha256 => self.0.1.sign_pss_sha256(&mut signature, msg)?,
      HashTy::Sha384 => self.0.1.sign_pss_sha384(&mut signature, msg)?,
      HashTy::Sha512 => self.0.1.sign_pss_sha512(&mut signature, msg)?,
    };
    Ok(SigningOutput::new(self.0.0, signature.into_vec().into_boxed_slice()))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    let vk = rsa::VerifyingKey::from_pkcs1_der(pk)?;
    match so.hash_ty() {
      HashTy::Sha256 => vk.verify_pss_sha256(so.signature(), msg)?,
      HashTy::Sha384 => vk.verify_pss_sha384(so.signature(), msg)?,
      HashTy::Sha512 => vk.verify_pss_sha512(so.signature(), msg)?,
    }
    Ok(())
  }
}
