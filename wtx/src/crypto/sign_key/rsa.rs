use crate::{
  crypto::{RsaPssSignKeySha256RustCrypto, RsaPssSignKeySha384RustCrypto, sign_key::SignKey},
  rng::CryptoRng,
};
use rsa::{RsaPrivateKey, pkcs8::DecodePrivateKey, pss::SigningKey};

impl SignKey for RsaPssSignKeySha256RustCrypto {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(SigningKey::new(RsaPrivateKey::from_pkcs8_der(bytes)?)))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(SigningKey::new(RsaPrivateKey::new(rng, 4096)?)))
  }
}

impl SignKey for RsaPssSignKeySha384RustCrypto {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(SigningKey::new(RsaPrivateKey::from_pkcs8_der(bytes)?)))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(SigningKey::new(RsaPrivateKey::new(rng, 4096)?)))
  }
}
