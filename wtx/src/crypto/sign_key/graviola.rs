use crate::crypto::{
  Ed25519SignKeyGraviola, P256SignKeyGraviola, P384SignKeyGraviola, RsaPssSignKeySha256Graviola,
  RsaPssSignKeySha384Graviola, sign_key::SignKey,
};
use graviola::signing::{
  ecdsa::{self, P256, P384},
  eddsa::Ed25519SigningKey,
  rsa,
};

impl SignKey for Ed25519SignKeyGraviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(Ed25519SigningKey::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for P256SignKeyGraviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(ecdsa::SigningKey::<P256>::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for P384SignKeyGraviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(ecdsa::SigningKey::<P384>::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for RsaPssSignKeySha256Graviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(rsa::SigningKey::from_pkcs8_der(bytes)?))
  }
}

impl SignKey for RsaPssSignKeySha384Graviola {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(rsa::SigningKey::from_pkcs8_der(bytes)?))
  }
}
