use crate::crypto::{
  Ed25519SignKeyRuco, P256SignKeyRuco, P384SignKeyRuco, RsaPssSignKeySha256Ruco,
  RsaPssSignKeySha384Ruco, sign_key::SignKey,
};
use pkcs8::DecodePrivateKey as _;

impl SignKey for Ed25519SignKeyRuco {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(ed25519_dalek::SigningKey::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for P256SignKeyRuco {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(p256::ecdsa::SigningKey::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for P384SignKeyRuco {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(p384::ecdsa::SigningKey::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for RsaPssSignKeySha256Ruco {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(rsa::pss::SigningKey::new(rsa::RsaPrivateKey::from_pkcs8_der(bytes)?)))
  }
}

impl SignKey for RsaPssSignKeySha384Ruco {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(rsa::pss::SigningKey::new(rsa::RsaPrivateKey::from_pkcs8_der(bytes)?)))
  }
}
