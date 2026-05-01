#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-openssl")]
mod openssl;
#[cfg(feature = "crypto-ring")]
mod ring;

use crate::{crypto::dummy_impl_call, misc::DefaultArray, rng::CryptoRng};
use core::marker::PhantomData;

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
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret>;

  /// New random ephemeral secret key
  fn ephemeral_secret_key<RNG>(rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng;

  /// Associated public key of an ephemeral secret
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey>;
}

/// Dummy [`Agreement`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AgreementDummy<ESK, PK, SS>(PhantomData<(ESK, PK, SS)>);

impl<ESK, PK, SS> Agreement for AgreementDummy<ESK, PK, SS>
where
  ESK: Default,
  PK: AsRef<[u8]> + DefaultArray,
  SS: AsRef<[u8]> + DefaultArray,
{
  type EphemeralSecretKey = ESK;
  type PublicKey = PK;
  type SharedSecret = SS;

  #[inline]
  fn diffie_hellman(_: Self::EphemeralSecretKey, _: &[u8]) -> crate::Result<Self::SharedSecret> {
    dummy_impl_call();
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    dummy_impl_call();
  }

  #[inline]
  fn public_key(_: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    dummy_impl_call();
  }
}
