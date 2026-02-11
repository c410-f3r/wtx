use crate::{crypto::AgreementAlgorithmTy, rng::CryptoRng};

/// Temporary single-use secret key.
pub trait Agreement: Sized {
  /// Ephemeral secret key
  type EphemeralSecretKey;
  /// Public key
  type PublicKey;
  /// Shared secret
  type SharedSecret;

  /// Generates a symmetric cryptographic key.
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    pk: &[u8],
  ) -> crate::Result<Self::SharedSecret>;

  /// New random instance
  fn ephemeral_secret_key<RNG>(
    &self,
    aat: AgreementAlgorithmTy,
    rng: &mut RNG,
  ) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng;

  /// Associated public key of an ephemeral secret
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey>;
}

impl Agreement for () {
  type EphemeralSecretKey = ();
  type SharedSecret = ();
  type PublicKey = [u8; 0];

  #[inline]
  fn diffie_hellman(
    &self,
    _: Self::EphemeralSecretKey,
    _: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    Ok(())
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(
    &self,
    _: AgreementAlgorithmTy,
    _: &mut RNG,
  ) -> crate::Result<Self::EphemeralSecretKey>
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
