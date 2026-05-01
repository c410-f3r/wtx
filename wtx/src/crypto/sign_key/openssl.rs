use crate::{
  crypto::{
    Ed25519SignKeyOpenssl, P256SignKeyOpenssl, P384SignKeyOpenssl, RsaPssSignKeySha256Openssl,
    RsaPssSignKeySha384Openssl, sign_key::SignKey,
  },
  rng::CryptoRng,
};
use openssl::{
  ec::{EcGroup, EcKey},
  nid::Nid,
  pkey::PKey,
  rsa::Rsa,
};

impl SignKey for Ed25519SignKeyOpenssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    let pkey = PKey::private_key_from_pkcs8(bytes).or_else(|_| {
      if bytes.len() == 32 {
        PKey::private_key_from_raw_bytes(bytes, openssl::pkey::Id::ED25519)
      } else {
        Err(openssl::error::ErrorStack::get())
      }
    });
    Ok(Self(pkey?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(PKey::generate_ed25519()?))
  }
}

impl SignKey for P256SignKeyOpenssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let key = EcKey::generate(&group)?;
    Ok(Self(PKey::from_ec_key(key)?))
  }
}

impl SignKey for P384SignKeyOpenssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let group = EcGroup::from_curve_name(Nid::SECP384R1)?;
    let key = EcKey::generate(&group)?;
    Ok(Self(PKey::from_ec_key(key)?))
  }
}

impl SignKey for RsaPssSignKeySha256Openssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(PKey::from_rsa(Rsa::generate(2048)?)?))
  }
}

impl SignKey for RsaPssSignKeySha384Openssl {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(PKey::private_key_from_pkcs8(bytes)?))
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(PKey::from_rsa(Rsa::generate(4096)?)?))
  }
}
