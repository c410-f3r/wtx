#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-ring")]
mod ring;
#[cfg(feature = "crypto-ruco")]
mod ruco;

/// A cryptographic secret usually composed by a secret key and a public key.
pub trait SignKey: Sized {
  /// New instance from a private key.
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self>;
}

/// Dummy [`SignKey`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignKeyDummy {}

impl SignKey for SignKeyDummy {
  #[inline]
  fn from_pkcs8(_: &[u8]) -> crate::Result<Self> {
    Ok(Self {})
  }
}
