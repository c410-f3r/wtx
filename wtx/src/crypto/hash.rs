use crate::{crypto::dummy_impl_call, misc::DefaultArray};
use core::marker::PhantomData;

#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-openssl")]
mod openssl;
#[cfg(feature = "crypto-ring")]
mod ring;

/// Maps data of arbitrary size into a fixed-size value.
pub trait Hash {
  /// Output array
  type Digest: AsRef<[u8]>;

  /// Computes the hash digest of the given `data` and writes the resulting
  /// fixed-size output into `buffer`.
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest;
}

/// Dummy [`Hash`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HashDummy<D>(PhantomData<D>);

impl<D> Hash for HashDummy<D>
where
  D: AsRef<[u8]> + DefaultArray,
{
  type Digest = D;

  #[inline]
  fn digest<'data>(_: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    dummy_impl_call();
  }
}
