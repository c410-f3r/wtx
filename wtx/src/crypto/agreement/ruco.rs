use crate::{
  crypto::{Agreement, AsRefWrapper, P256Ruco, P384Ruco, X25519Ruco},
  rng::CryptoRng,
};
use crypto_common::Generate as _;

impl Agreement for P256Ruco {
  type PublicKey = p256::Sec1Point;
  type SharedSecret = AsRefWrapper<p256::ecdh::SharedSecret>;

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    Ok(AsRefWrapper(
      self.0.diffie_hellman(&p256::PublicKey::from_sec1_bytes(other_participant_pk)?),
    ))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    match p256::ecdh::EphemeralSecret::try_generate_from_rng(rng) {
      Ok(el) => Ok(Self(el)),
    }
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    Ok(p256::Sec1Point::from(&self.0.public_key()))
  }
}

impl AsRef<[u8]> for AsRefWrapper<p256::ecdh::SharedSecret> {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    self.0.raw_secret_bytes()
  }
}

impl Agreement for P384Ruco {
  type PublicKey = p384::Sec1Point;
  type SharedSecret = AsRefWrapper<p384::ecdh::SharedSecret>;

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    Ok(AsRefWrapper(
      self.0.diffie_hellman(&p384::PublicKey::from_sec1_bytes(other_participant_pk)?),
    ))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    match p384::ecdh::EphemeralSecret::try_generate_from_rng(rng) {
      Ok(el) => Ok(Self(el)),
    }
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    Ok(p384::Sec1Point::from(self.0.public_key()))
  }
}

impl AsRef<[u8]> for AsRefWrapper<p384::ecdh::SharedSecret> {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    self.0.raw_secret_bytes()
  }
}

impl Agreement for X25519Ruco {
  type PublicKey = x25519_dalek::PublicKey;
  type SharedSecret = x25519_dalek::SharedSecret;

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    let array: [u8; 32] = other_participant_pk.try_into()?;
    Ok(self.0.diffie_hellman(&x25519_dalek::PublicKey::from(array)))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(x25519_dalek::EphemeralSecret::random_from_rng(rng)))
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    Ok(x25519_dalek::PublicKey::from(&self.0))
  }
}

impl AsRef<[u8]> for AsRefWrapper<x25519_dalek::SharedSecret> {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    self.0.as_bytes()
  }
}
