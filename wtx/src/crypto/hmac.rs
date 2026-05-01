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

/// Keyed-hash message authentication code.
pub trait Hmac: Sized {
  /// Fixed-size output
  type Digest: AsRef<[u8]>;

  /// Creates a new instance from the given secret key.
  fn from_key(key: &[u8]) -> crate::Result<Self>;

  /// Feeds additional data into the MAC computation.
  fn update(&mut self, data: &[u8]);

  /// Finalizes the computation and returns the resulting digest.
  fn digest(self) -> Self::Digest;

  /// Finalizes the computation and verifies the result against the provided tag.
  fn verify(self, tag: &[u8]) -> crate::Result<()>;
}

/// Dummy [`Hmac`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HmacDummy<D>(PhantomData<D>);

impl<D> Hmac for HmacDummy<D>
where
  D: AsRef<[u8]> + DefaultArray,
{
  type Digest = D;

  #[inline]
  fn from_key(_: &[u8]) -> crate::Result<Self> {
    dummy_impl_call();
  }

  #[inline]
  fn update(&mut self, _: &[u8]) {}

  #[inline]
  fn digest(self) -> Self::Digest {
    dummy_impl_call();
  }

  #[inline]
  fn verify(self, _: &[u8]) -> crate::Result<()> {
    dummy_impl_call();
  }
}
