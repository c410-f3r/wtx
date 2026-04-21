#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "ed25519-dalek")]
mod ed25519_dalek;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "p256")]
mod p256;
#[cfg(feature = "p384")]
mod p384;
#[cfg(feature = "crypto-ring")]
mod ring;
#[cfg(feature = "rsa")]
mod rsa;

use crate::rng::CryptoRng;

/// A cryptographic secret usually composed by a secret key and a public key.
pub trait SignKey: Sized {
  /// New instance from a private key.
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self>;

  /// Generates a new instance with a random private key.
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng;
}

/// Stub [`SignKey`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignKeyStub;

impl SignKey for SignKeyStub {
  #[inline]
  fn from_pkcs8(_: &[u8]) -> crate::Result<Self> {
    Ok(Self)
  }

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self)
  }
}
