use crate::{
  collection::ArrayVectorU8,
  crypto::{AgreementAlgorithmTy, MAX_PK_LEN},
  rng::CryptoRng,
};

/// Temporary single-use secret key.
pub trait Agreement: Sized {
  type EphemeralSecretKey;
  type SharedSecret;

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
  fn public_key(
    &self,
    esk: &Self::EphemeralSecretKey,
  ) -> crate::Result<ArrayVectorU8<u8, MAX_PK_LEN>>;
}

impl Agreement for () {
  type EphemeralSecretKey = ();
  type SharedSecret = ();

  fn diffie_hellman(
    &self,
    _: Self::EphemeralSecretKey,
    _: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    Ok(())
  }

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

  fn public_key(
    &self,
    _: &Self::EphemeralSecretKey,
  ) -> crate::Result<ArrayVectorU8<u8, MAX_PK_LEN>> {
    Ok(ArrayVectorU8::new())
  }
}
