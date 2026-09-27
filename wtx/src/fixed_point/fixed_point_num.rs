use core::num::{
  NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64,
};

/// Underlying number primitive used as storage.
pub trait FixedPointNum: Clone + Copy + Eq + Ord + PartialEq + PartialOrd {
  /// Instance with the minimum value
  const MIN: Self;
  /// Instance with the minimum value in `i128`;
  const MIN_I128: i128;
  /// Instance with the maximum value
  const MAX: Self;
  /// Instance with the maximum value in `u128`
  const MAX_U128: u128;
}

macro_rules! fixed_point_num {
  ($( ($ty:ty, ($min:expr, $max:expr)) ),* $(,)?) => {
    $(
      impl FixedPointNum for $ty {
        const MIN: Self = <$ty>::MIN;
        const MIN_I128: i128 = $min;
        const MAX: Self = <$ty>::MAX;
        const MAX_U128: u128 = $max;
      }
    )*
  };
}

fixed_point_num!(
  // Signed
  (i8, (-128, 127)),
  (i16, (-32_768, 32_767)),
  (i32, (-2_147_483_648, 2_147_483_647)),
  (i64, (-9_223_372_036_854_775_808, 9_223_372_036_854_775_807)),
  (NonZeroI8, (-128, 127)),
  (NonZeroI16, (-32_768, 32_767)),
  (NonZeroI32, (-2_147_483_648, 2_147_483_647)),
  (NonZeroI64, (-9_223_372_036_854_775_808, 9_223_372_036_854_775_807)),
  // Unsigned
  (u8, (0, 255)),
  (u16, (0, 65_535)),
  (u32, (0, 4_294_967_295)),
  (u64, (0, 18_446_744_073_709_551_615)),
  (NonZeroU8, (1, 255)),
  (NonZeroU16, (1, 65_535)),
  (NonZeroU32, (1, 4_294_967_295)),
  (NonZeroU64, (1, 18_446_744_073_709_551_615)),
);
