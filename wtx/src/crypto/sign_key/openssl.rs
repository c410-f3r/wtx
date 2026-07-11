use crate::crypto::{
  Ed25519SignKeyOpenssl, P256SignKeyOpenssl, P384SignKeyOpenssl, RsaPssSignKeySha256Openssl,
  RsaPssSignKeySha384Openssl, sign_key::SignKey,
};
use openssl::pkey::PKey;

impl SignKey for Ed25519SignKeyOpenssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }
}

impl SignKey for P256SignKeyOpenssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }
}

impl SignKey for P384SignKeyOpenssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }
}

impl SignKey for RsaPssSignKeySha256Openssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }
}

impl SignKey for RsaPssSignKeySha384Openssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }
}
