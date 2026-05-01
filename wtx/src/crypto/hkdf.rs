use crate::{crypto::dummy_impl_call, misc::DefaultArray};
use core::marker::PhantomData;

#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
pub(crate) mod global;
#[cfg(feature = "crypto-ring")]
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

/// Dummy [`Hkdf`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HkdfDummy<D>(PhantomData<D>);

impl<D> Hkdf for HkdfDummy<D>
where
  D: DefaultArray,
{
  type Digest = D;

  #[inline]
  fn extract(_: Option<&[u8]>, _: &[u8]) -> (Self::Digest, Self) {
    dummy_impl_call();
  }

  #[inline]
  fn from_prk(_: &[u8]) -> crate::Result<Self> {
    dummy_impl_call();
  }

  #[inline]
  fn compute<'data>(
    _: impl IntoIterator<Item = &'data [u8]>,
    _: &[u8],
  ) -> crate::Result<Self::Digest> {
    dummy_impl_call();
  }

  #[inline]
  fn expand(&self, _: &[u8], _: &mut [u8]) -> crate::Result<()> {
    dummy_impl_call();
  }
}
