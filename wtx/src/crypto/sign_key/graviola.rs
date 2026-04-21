use crate::{
  crypto::{
    CryptoError, Ed25519SignKeyGraviola, P256SignKeyGraviola, P384SignKeyGraviola,
    RsaPssSignKeySha256Graviola, RsaPssSignKeySha384Graviola, sign_key::SignKey,
  },
  rng::CryptoRng,
};
use graviola::signing::{
  ecdsa::{self, P256, P384},
  eddsa::Ed25519SigningKey,
  rsa::{self, KeySize},
};

impl SignKey for Ed25519SignKeyGraviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(Ed25519SigningKey::from_pkcs8_der(bytes)?))
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

impl SignKey for P256SignKeyGraviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(ecdsa::SigningKey::<P256>::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut secret = [0; 32];
    rng.fill_slice(&mut secret);
    Self::from_pkcs8(&secret)
  }
}

impl SignKey for P384SignKeyGraviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(ecdsa::SigningKey::<P384>::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut secret = [0; 48];
    rng.fill_slice(&mut secret);
    Self::from_pkcs8(&secret)
  }
}

impl SignKey for RsaPssSignKeySha256Graviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(rsa::SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(rsa::SigningKey::generate(KeySize::Rsa2048).map_err(|_| CryptoError::SignKeyError)?))
  }
}

impl SignKey for RsaPssSignKeySha384Graviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(rsa::SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(rsa::SigningKey::generate(KeySize::Rsa4096).map_err(|_| CryptoError::SignKeyError)?))
  }
}
