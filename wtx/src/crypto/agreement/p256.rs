use crate::{
  crypto::{Agreement, AsRefWrapper, P256RustCrypto},
  rng::CryptoRng,
};
use crypto_common::Generate;
use p256::{
  PublicKey, Sec1Point,
  ecdh::{EphemeralSecret, SharedSecret},
};

impl Agreement for P256RustCrypto {
  type EphemeralSecretKey = EphemeralSecret;
  type PublicKey = Sec1Point;
  type SharedSecret = AsRefWrapper<SharedSecret>;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    Ok(AsRefWrapper(esk.diffie_hellman(&PublicKey::from_sec1_bytes(other_participant_pk)?)))
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(match EphemeralSecret::try_generate_from_rng(rng) {
      Ok(el) => el,
    })
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(Sec1Point::from(esk.public_key()))
  }
}

impl AsRef<[u8]> for AsRefWrapper<SharedSecret> {
  fn as_ref(&self) -> &[u8] {
    self.0.raw_secret_bytes()
  }
}
