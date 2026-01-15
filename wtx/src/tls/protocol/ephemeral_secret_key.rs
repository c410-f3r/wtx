use crate::{
  collection::ArrayVectorU8,
  rng::CryptoRng,
  tls::{MAX_PK_LEN, NamedGroup},
};

/// Temporary single-use secret key.
pub trait EphemeralSecretKey: Sized {
  type SharedSecret;

  /// New random instance
  fn random<RNG>(ng: NamedGroup, rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng;

  fn diffie_hellman(&self, pk: &[u8]) -> Self::SharedSecret;

  /// Associated public key
  fn public_key(&self) -> crate::Result<ArrayVectorU8<u8, MAX_PK_LEN>>;
}

impl EphemeralSecretKey for () {
  type SharedSecret = ();

  fn random<RNG>(_: NamedGroup, _: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(())
  }

  fn diffie_hellman(&self, _: &[u8]) -> Self::SharedSecret {
    ()
  }

  fn public_key(&self) -> crate::Result<ArrayVectorU8<u8, MAX_PK_LEN>> {
    Ok(ArrayVectorU8::new())
  }
}
