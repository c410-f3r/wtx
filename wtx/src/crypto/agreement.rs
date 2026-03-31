#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "graviola")]
mod graviola;
#[cfg(feature = "p256")]
mod p256;
#[cfg(feature = "p384")]
mod p384;
#[cfg(feature = "ring")]
mod ring;
#[cfg(feature = "x25519-dalek")]
mod x25519_dalek;

use crate::rng::CryptoRng;

/// Temporary single-use secret key.
pub trait Agreement: Sized {
  /// Ephemeral secret key
  type EphemeralSecretKey;
  /// Public key
  type PublicKey: AsRef<[u8]>;
  /// Shared secret
  type SharedSecret: AsRef<[u8]>;

  /// Generates a symmetric cryptographic key.
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret>;

  /// New random ephemeral secret key
  fn ephemeral_secret_key<RNG>(&self, rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng;

  /// Associated public key of an ephemeral secret
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey>;
}

impl Agreement for () {
  type EphemeralSecretKey = ();
  type SharedSecret = [u8; 0];
  type PublicKey = [u8; 0];

  #[inline]
  fn diffie_hellman(
    &self,
    _: Self::EphemeralSecretKey,
    _: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    Ok([])
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(&self, _: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(())
  }

  #[inline]
  fn public_key(&self, _: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok([0; 0])
  }
}
