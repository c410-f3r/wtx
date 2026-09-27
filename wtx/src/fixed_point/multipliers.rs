use crate::{
  fixed_point::{FixedPoint, FixedPointNum, MAX_DECIMALS},
  misc::int_conv::i32i128,
};

impl<T, const N: usize> FixedPoint<T, N>
where
  T: FixedPointNum,
{
  pub(crate) const MULTIPLIER_PCT_I32: i32 = {
    assert!(N <= MAX_DECIMALS);
    #[expect(
      clippy::as_conversions,
      clippy::cast_possible_truncation,
      reason = "`i32` can hold `10^8`"
    )]
    let exp = N.checked_sub(2).unwrap() as u32;
    10i32.pow(exp)
  };
  pub(crate) const MULTIPLIER_PCT_U32: u32 = Self::MULTIPLIER_PCT_I32.cast_unsigned();
  pub(crate) const MULTIPLIER_PCT_I128: i128 = i32i128(Self::MULTIPLIER_PCT_I32);
  pub(crate) const MULTIPLIER_PCT_U128: u128 = Self::MULTIPLIER_PCT_I128.cast_unsigned();

  pub(crate) const MULTIPLIER_BPS_I32: i32 = {
    assert!(N <= MAX_DECIMALS);
    #[expect(
      clippy::as_conversions,
      clippy::cast_possible_truncation,
      reason = "`i32` can hold `10^6`"
    )]
    let exp = N.checked_sub(4).unwrap() as u32;
    10i32.pow(exp)
  };
  pub(crate) const MULTIPLIER_BPS_U32: u32 = Self::MULTIPLIER_BPS_I32.cast_unsigned();
  pub(crate) const MULTIPLIER_BPS_I128: i128 = i32i128(Self::MULTIPLIER_BPS_I32);
  pub(crate) const MULTIPLIER_BPS_U128: u128 = Self::MULTIPLIER_BPS_I128.cast_unsigned();
}
