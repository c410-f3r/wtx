use crate::rng::Rng;

/// A random number generator that can be explicitly seeded.
pub trait SeedableRng: Rng {
  /// Creates a new instance based on another RNG.
  fn from_rng<R>(rng: &mut R) -> Self
  where
    R: Rng;
}
