use crate::misc::{Rng, Usize};

/// Allows the creation of random instances.
pub trait FromRng<RNG>
where
  RNG: Rng,
{
  /// Creates a new instance based on `rng`.
  fn from_rng(rng: &mut RNG) -> Self;
}

impl<RNG> FromRng<RNG> for u8
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    rng.u8()
  }
}

impl<RNG> FromRng<RNG> for usize
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    Usize::from_u64(u64::from_be_bytes(rng.u8_8()))
      .unwrap_or_else(|| Usize::from_u32(u32::from_be_bytes(rng.u8_4())))
      .into_usize()
  }
}
