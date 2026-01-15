use crate::{misc::LeaseMut, rng::Rng};

/// A random number generator that can be explicitly seeded.
pub trait SeedableRng: Sized {
  /// Number used to construct instances
  type Seed: Clone + Default + LeaseMut<[u8]>;

  /// Creates a new instance based on the entropy provided by the `getrandom` dependency.
  #[cfg(feature = "getrandom")]
  #[inline]
  fn from_os() -> crate::Result<Self> {
    let mut seed = Self::Seed::default();
    getrandom::fill(seed.lease_mut())?;
    Self::from_seed(seed)
  }

  /// Creates a new instance based on the entropy provided by `std::random`.
  #[cfg(all(feature = "nightly", feature = "std"))]
  #[inline]
  fn from_random() -> crate::Result<Self> {
    use core::random::RandomSource;
    let mut seed = Self::Seed::default();
    std::random::DefaultRandomSource.fill_bytes(seed.lease_mut());
    Self::from_seed(seed)
  }

  /// Creates a new instance based on another RNG.
  #[inline]
  fn from_rng<R>(rng: &mut R) -> crate::Result<Self>
  where
    R: Rng,
  {
    let mut seed = Self::Seed::default();
    rng.fill_slice(seed.lease_mut());
    Self::from_seed(seed)
  }

  /// Creates a new instance based on the provided seed.
  fn from_seed(seed: Self::Seed) -> crate::Result<Self>;
}
