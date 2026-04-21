use crate::{
  crypto::{
    CryptoError, Ed25519SignKeyRing, P256SignKeyRing, P384SignKeyRing, RsaPssSignKeySha256Ring,
    RsaPssSignKeySha384Ring, sign_key::SignKey,
  },
  rng::CryptoRng,
};
use ring::{
  rand::SystemRandom,
  signature::{
    ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P384_SHA384_ASN1_SIGNING, EcdsaKeyPair, Ed25519KeyPair,
    RsaKeyPair,
  },
};

impl SignKey for Ed25519SignKeyRing {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(
      Ed25519KeyPair::from_pkcs8_maybe_unchecked(bytes)
        .map_err(|_err| CryptoError::SignKeyError)?,
    ))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut seed = [0u8; 32];
    rng.fill_slice(&mut seed);
    Self::from_pkcs8(&seed)
  }
}

impl SignKey for P256SignKeyRing {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    let rng = SystemRandom::new();
    Ok(Self(
      EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, bytes, &rng)
        .map_err(|_| CryptoError::SignKeyError)?,
    ))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &SystemRandom::new())
      .map_err(|_| CryptoError::SignKeyError)?;
    Self::from_pkcs8(pkcs8.as_ref())
  }
}

impl SignKey for P384SignKeyRing {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(
      EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_ASN1_SIGNING, bytes, &SystemRandom::new())
        .map_err(|_| CryptoError::SignKeyError)?,
    ))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P384_SHA384_ASN1_SIGNING, &SystemRandom::new())
      .map_err(|_| CryptoError::SignKeyError)?;
    Self::from_pkcs8(pkcs8.as_ref())
  }
}

impl SignKey for RsaPssSignKeySha256Ring {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(RsaKeyPair::from_pkcs8(bytes).map_err(|_| CryptoError::SignKeyError)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Err(crate::Error::UnsupportedOperation)
  }
}

impl SignKey for RsaPssSignKeySha384Ring {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(RsaKeyPair::from_pkcs8(bytes).map_err(|_| CryptoError::SignKeyError)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Err(crate::Error::UnsupportedOperation)
  }
}
