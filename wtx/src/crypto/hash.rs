use crate::misc::DefaultArray;
use core::marker::PhantomData;

#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-ring")]
mod ring;
#[cfg(feature = "sha1")]
mod sha1;
#[cfg(feature = "sha2")]
mod sha2;

/// Maps data of arbitrary size into a fixed-size value.
pub trait Hash {
  /// Output array
  type Digest: AsRef<[u8]>;

  /// Computes the hash digest of the given `data` and writes the resulting
  /// fixed-size output into `buffer`.
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest;
}

/// Stub [`Hash`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HashStub<D>(PhantomData<D>);

impl<D> Hash for HashStub<D>
where
  D: AsRef<[u8]> + DefaultArray,
{
  type Digest = D;

  #[inline]
  fn digest<'data>(_: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Self::Digest::default_array()
  }
}
