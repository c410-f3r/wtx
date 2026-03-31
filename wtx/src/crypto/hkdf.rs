#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "hkdf")]
mod hkdf;
#[cfg(feature = "ring")]
mod ring;

/// HMAC-based Key Derivation Function
pub trait Hkdf: Sized {
  /// Output array
  type Digest;

  /// The HKDF-Extract operation from the RFC-5869
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self);

  /// Creates a new instance from an already cryptographically strong pseudorandom key.
  fn from_prk(prk: &[u8]) -> crate::Result<Self>;

  /// Performs a one-shot HMAC
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest>;

  /// The HKDF-Expand operation from the RFC-5869
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()>;
}

impl Hkdf for () {
  type Digest = [u8; 0];

  #[inline]
  fn extract(_: Option<&[u8]>, _: &[u8]) -> (Self::Digest, Self) {
    ([0; 0], ())
  }

  #[inline]
  fn from_prk(_: &[u8]) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn compute<'data>(
    _: impl IntoIterator<Item = &'data [u8]>,
    _: &[u8],
  ) -> crate::Result<Self::Digest> {
    Ok([0; 0])
  }

  #[inline]
  fn expand(&self, _: &[u8], _: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }
}
