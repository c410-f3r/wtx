#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-openssl")]
mod openssl;
#[cfg(feature = "crypto-ring")]
mod ring;

use crate::{crypto::dummy_impl_call, rng::CryptoRng};

/// A cryptographic secret usually composed by a secret key and a public key.
pub trait SignKey: Sized {
  /// New instance from a private key.
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self>;

  /// Generates a new instance with a random private key.
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng;
}

/// Dummy [`SignKey`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignKeyDummy;

impl SignKey for SignKeyDummy {
  #[inline]
  fn from_pkcs8(_: &[u8]) -> crate::Result<Self> {
    dummy_impl_call();
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    dummy_impl_call();
  }
}
