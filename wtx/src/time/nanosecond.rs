use crate::time::TimeError;

/// This particular structure can represent at most one second in nanoseconds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Nanosecond(u32);

impl Nanosecond {
  /// Instance with the maximum allowed value of `999_999_999`
  pub const MAX: Self = Self(999_999_999);
  /// Instance with the minimum allowed value of `0`
  pub const MIN: Self = Self(0);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u32) -> Result<Self, TimeError> {
    if num > 999_999_999 {
      return Err(TimeError::InvalidNanosecond { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u32 {
    self.0
  }
}
