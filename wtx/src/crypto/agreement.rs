#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-ring")]
mod ring;
#[cfg(feature = "crypto-ruco")]
mod ruco;

use crate::{crypto::dummy_crypto_call, misc::DefaultArray, rng::CryptoRng};
use core::marker::PhantomData;

/// Temporary single-use secret key.
pub trait Agreement: Sized {
  /// Public key
  type PublicKey: AsRef<[u8]>;
  /// Shared secret
  type SharedSecret: AsRef<[u8]>;

  /// Generates a symmetric cryptographic key.
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret>;

  /// New random ephemeral secret key
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng;

  /// Associated public key of an ephemeral secret
  fn public_key(&self) -> crate::Result<Self::PublicKey>;
}

/// Dummy [`Agreement`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AgreementDummy<PK, SS>(PhantomData<(PK, SS)>);

impl<PK, SS> Agreement for AgreementDummy<PK, SS>
where
  PK: AsRef<[u8]> + DefaultArray,
  SS: AsRef<[u8]> + DefaultArray,
{
  type PublicKey = PK;
  type SharedSecret = SS;

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(PhantomData))
  }

  #[inline]
  fn diffie_hellman(self, _: &[u8]) -> crate::Result<Self::SharedSecret> {
    dummy_crypto_call();
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    dummy_crypto_call();
  }
}
