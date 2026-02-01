use crate::{misc::Usize, rng::Rng};

/// Allows the creation of random instances.
pub trait FromRng<RNG>
where
  RNG: Rng,
{
  /// Creates a new instance based on `rng`.
  fn from_rng(rng: &mut RNG) -> Self;
}

impl<RNG> FromRng<RNG> for bool
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    !matches!(u8::from_rng(rng) % 2, 0)
  }
}

impl<RNG> FromRng<RNG> for isize
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    usize::from_rng(rng).cast_signed()
  }
}

impl<RNG> FromRng<RNG> for usize
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    #[cfg(target_pointer_width = "64")]
    return Usize::from_u64(u64::from_be_bytes(rng.u8_8())).into_usize();
    #[cfg(not(target_pointer_width = "64"))]
    return Usize::from_u32(u32::from_be_bytes(rng.u8_4())).into_usize();
  }
}

#[cfg(feature = "rust_decimal")]
impl<RNG> FromRng<RNG> for rust_decimal::Decimal
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    rust_decimal::Decimal::from_parts(
      u32::from_rng(rng),
      u32::from_rng(rng),
      u32::from_rng(rng),
      bool::from_rng(rng),
      u32::from_rng(rng),
    )
  }
}

macro_rules! implement {
  ($(($ty:ty, $from:pat, $method:ident, $to:expr)),* $(,)?) => {
    $(
      impl<RNG> FromRng<RNG> for $ty
      where
        RNG: Rng,
      {
        #[inline]
        fn from_rng(rng: &mut RNG) -> Self {
          let $from = rng.$method();
          <$ty>::from_be_bytes($to)
        }
      }
    )*
  };
}

implement!(
  (f32, [a, b, c, d], u8_4, [a, b, c, d]),
  (f64, [a, b, c, d, e, f, g, h], u8_8, [a, b, c, d, e, f, g, h]),
  //
  (i8, [a, _, _, _], u8_4, [a]),
  (i16, [a, b, _, _], u8_4, [a, b]),
  (i32, [a, b, c, d], u8_4, [a, b, c, d]),
  (i64, [a, b, c, d, e, f, g, h], u8_8, [a, b, c, d, e, f, g, h]),
  (
    i128,
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p],
    u8_16,
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
  ),
  //
  (u8, [a, _, _, _], u8_4, [a]),
  (u16, [a, b, _, _], u8_4, [a, b]),
  (u32, [a, b, c, d], u8_4, [a, b, c, d]),
  (u64, [a, b, c, d, e, f, g, h], u8_8, [a, b, c, d, e, f, g, h]),
  (
    u128,
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p],
    u8_16,
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
  ),
);
