#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "graviola")]
mod graviola;
#[cfg(feature = "ring")]
mod ring;
#[cfg(feature = "sha2")]
mod sha2;

/// Maps data of arbitrary size into a fixed-size value.
pub trait Hash {
  /// Output array
  type Digest: AsRef<[u8]>;

  /// Computes the hash digest of the given `data` and writes the resulting
  /// fixed-size output into `buffer`.
  fn digest(data: &[u8]) -> Self::Digest;
}

impl Hash for () {
  type Digest = [u8; 0];

  #[inline]
  fn digest(_: &[u8]) -> Self::Digest {
    [0; 0]
  }
}
