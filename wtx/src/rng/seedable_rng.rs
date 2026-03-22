use crate::{
  misc::LeaseMut as _,
  rng::{CryptoSeedableRng, Rng, Xorshift64, simple_seed},
};

/// Non-crypto version of [`CryptoSeedableRng`].
pub trait SeedableRng: CryptoSeedableRng {
  /// Creates a new instance based on the entropy provided by [`simple_seed`].
  #[inline]
  fn from_simple_seed() -> crate::Result<Self> {
    let mut seed = Self::Seed::default();
    Xorshift64::from(simple_seed()).fill_slice(seed.lease_mut());
    Self::from_seed(seed)
  }

  /// Creates a new instance based on another non-crypto RNG.
  #[inline]
  fn from_rng<R>(rng: &mut R) -> crate::Result<Self>
  where
    R: Rng,
  {
    let mut seed = Self::Seed::default();
    rng.fill_slice(seed.lease_mut());
    Self::from_seed(seed)
  }
}
