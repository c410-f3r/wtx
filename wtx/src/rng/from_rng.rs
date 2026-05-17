use crate::{misc::Usize, rng::Rng};

/// Allows the creation of random instances.
pub trait FromRng<RNG>
where
  RNG: Rng,
{
  /// Creates b0 new instance based on `rng`.
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
  (f32, [b0, b1, b2, b3], u8_4, [b0, b1, b2, b3]),
  (f64, [b0, b1, b2, b3, b4, b5, b6, b7], u8_8, [b0, b1, b2, b3, b4, b5, b6, b7]),
  (i8, [b0, _, _, _], u8_4, [b0]),
  (i16, [b0, b1, _, _], u8_4, [b0, b1]),
  (i32, [b0, b1, b2, b3], u8_4, [b0, b1, b2, b3]),
  (i64, [b0, b1, b2, b3, b4, b5, b6, b7], u8_8, [b0, b1, b2, b3, b4, b5, b6, b7]),
  (
    i128,
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15],
    u8_16,
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15]
  ),
  (u8, [b0, _, _, _], u8_4, [b0]),
  (u16, [b0, b1, _, _], u8_4, [b0, b1]),
  (u32, [b0, b1, b2, b3], u8_4, [b0, b1, b2, b3]),
  (u64, [b0, b1, b2, b3, b4, b5, b6, b7], u8_8, [b0, b1, b2, b3, b4, b5, b6, b7]),
  (
    u128,
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15],
    u8_16,
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15]
  ),
);
