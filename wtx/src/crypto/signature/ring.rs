use crate::{
  collection::Vector,
  crypto::{
    CryptoError, Ed25519Ring, Ed25519SignKeyRing, P256Ring, P256SignKeyRing, P384Ring,
    P384SignKeyRing, RsaPssRsaeSha256Ring, RsaPssRsaeSha384Ring, RsaPssSignKeySha256Ring,
    RsaPssSignKeySha384Ring, Signature,
  },
  rng::CryptoRng,
};
use ring::{
  rand::SystemRandom,
  signature::{
    ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1, ED25519, RSA_PSS_2048_8192_SHA256,
    RSA_PSS_2048_8192_SHA384, RSA_PSS_SHA256, RSA_PSS_SHA384, UnparsedPublicKey,
    VerificationAlgorithm,
  },
};

impl Signature for P256Ring {
  type SignKey = P256SignKeyRing;
  type SignOutput = ring::signature::Signature;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    sign_key.0.sign(&SystemRandom::new(), msg).map_err(|_err| CryptoError::SignatureError.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    validate_signature(&ECDSA_P256_SHA256_ASN1, pk, msg, signature)
  }
}

impl Signature for P384Ring {
  type SignKey = P384SignKeyRing;
  type SignOutput = ring::signature::Signature;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    sign_key.0.sign(&SystemRandom::new(), msg).map_err(|_err| CryptoError::SignatureError.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    validate_signature(&ECDSA_P384_SHA384_ASN1, pk, msg, signature)
  }
}

impl Signature for Ed25519Ring {
  type SignKey = Ed25519SignKeyRing;
  type SignOutput = ring::signature::Signature;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    Ok(sign_key.0.sign(msg))
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    validate_signature(&ED25519, pk, msg, signature)
  }
}

impl Signature for RsaPssRsaeSha256Ring {
  type SignKey = RsaPssSignKeySha256Ring;
  type SignOutput = Vector<u8>;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let rng = SystemRandom::new();
    let mut signature = Vector::from_vec(alloc::vec![0; sign_key.0.public().modulus_len()]);
    sign_key
      .0
      .sign(&RSA_PSS_SHA256, &rng, msg, &mut signature)
      .map_err(|_err| CryptoError::SignatureError)?;
    Ok(signature)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    validate_signature(&RSA_PSS_2048_8192_SHA256, pk, msg, signature)
  }
}

impl Signature for RsaPssRsaeSha384Ring {
  type SignKey = RsaPssSignKeySha384Ring;
  type SignOutput = Vector<u8>;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let rng = SystemRandom::new();
    let mut signature = Vector::from_vec(alloc::vec![0; sign_key.0.public().modulus_len()]);
    sign_key
      .0
      .sign(&RSA_PSS_SHA384, &rng, msg, &mut signature)
      .map_err(|_err| CryptoError::SignatureError)?;
    Ok(signature)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    validate_signature(&RSA_PSS_2048_8192_SHA384, pk, msg, signature)
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
