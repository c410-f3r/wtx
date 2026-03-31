use crate::{
  crypto::{Agreement, X25519RustCrypto},
  rng::CryptoRng,
};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

impl Agreement for X25519RustCrypto {
  type EphemeralSecretKey = EphemeralSecret;
  type PublicKey = PublicKey;
  type SharedSecret = SharedSecret;

  #[inline]
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let array: [u8; 32] = other_participant_pk.try_into()?;
    Ok(esk.diffie_hellman(&PublicKey::from(array)))
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(&self, rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(EphemeralSecret::random_from_rng(rng))
  }

  #[inline]
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(PublicKey::from(esk))
  }
}
