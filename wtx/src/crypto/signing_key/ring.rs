use crate::{
  collections::Vector,
  crypto::{
    CryptoError, EcdsaP256SigningKeyRing, EcdsaP384SigningKeyRing, Ed25519SigningKeyRing, HashTy,
    RsaPkcs1SigningKeyRing, RsaPssSigningKeyRing, SigningOutput, signing_key::SigningKey,
  },
  rng::CryptoRng,
};
use ring::{
  rand::SystemRandom,
  signature::{
    ECDSA_P256_SHA256_ASN1, ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P384_SHA384_ASN1,
    ECDSA_P384_SHA384_ASN1_SIGNING, ED25519, EcdsaKeyPair, Ed25519KeyPair, RsaKeyPair,
    UnparsedPublicKey, VerificationAlgorithm,
  },
};

impl SigningKey for EcdsaP256SigningKeyRing {
  type Signature = ring::signature::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(
      EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, bytes, &SystemRandom::new())
        .map_err(|_err| CryptoError::SigningKeyError)?,
    ))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let signature =
      self.0.sign(&SystemRandom::new(), msg).map_err(|_err| CryptoError::SignatureError)?;
    Ok(SigningOutput::new(HashTy::Sha256, signature))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    validate_signature(&ECDSA_P256_SHA256_ASN1, pk, msg, so.signature())
  }
}

impl SigningKey for EcdsaP384SigningKeyRing {
  type Signature = ring::signature::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(
      EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_ASN1_SIGNING, bytes, &SystemRandom::new())
        .map_err(|_err| CryptoError::SigningKeyError)?,
    ))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let signature =
      self.0.sign(&SystemRandom::new(), msg).map_err(|_err| CryptoError::SignatureError)?;
    Ok(SigningOutput::new(HashTy::Sha384, signature))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    validate_signature(&ECDSA_P384_SHA384_ASN1, pk, msg, so.signature())
  }
}

impl SigningKey for Ed25519SigningKeyRing {
  type Signature = ring::signature::Signature;

  #[inline]
  fn from_pkcs8(bytes: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(
      Ed25519KeyPair::from_pkcs8_maybe_unchecked(bytes)
        .map_err(|_err| CryptoError::SigningKeyError)?,
    ))
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
    validate_signature(&ED25519, pk, msg, so.signature())
  }
}

impl SigningKey for RsaPkcs1SigningKeyRing {
  type Signature = Vector<u8>;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self((hash_ty, RsaKeyPair::from_pkcs8(bytes).map_err(|_err| CryptoError::SigningKeyError)?)))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let mut signature = Vector::from_vec(alloc::vec![0; self.0.1.public().modulus_len()]);
    self
      .0
      .1
      .sign(self.0.0.rsa_pkcs1_enc_ring(), &SystemRandom::new(), msg, &mut signature)
      .map_err(|_err| CryptoError::SignatureError)?;
    Ok(SigningOutput::new(self.0.0, signature))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    validate_signature(so.hash_ty().rsa_pkcs1_params_ring(), pk, msg, so.signature())
  }
}

impl SigningKey for RsaPssSigningKeyRing {
  type Signature = Vector<u8>;

  #[inline]
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self> {
    Ok(Self((hash_ty, RsaKeyPair::from_pkcs8(bytes).map_err(|_err| CryptoError::SigningKeyError)?)))
  }

  #[inline]
  fn sign<RNG>(&self, msg: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    let mut signature = Vector::from_vec(alloc::vec![0; self.0.1.public().modulus_len()]);
    self
      .0
      .1
      .sign(self.0.0.rsa_pss_enc_ring(), &SystemRandom::new(), msg, &mut signature)
      .map_err(|_err| CryptoError::SignatureError)?;
    Ok(SigningOutput::new(self.0.0, signature))
  }

  #[inline]
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()> {
    validate_signature(so.hash_ty().rsa_pss_params_ring(), pk, msg, so.signature())
  }
}

#[inline]
fn validate_signature(
  alg: &'static dyn VerificationAlgorithm,
  pk: &[u8],
  msg: &[u8],
  signature: &[u8],
) -> crate::Result<()> {
  UnparsedPublicKey::new(alg, pk)
    .verify(msg, signature)
    .map_err(|_err| CryptoError::SignatureError.into())
}
