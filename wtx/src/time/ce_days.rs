use crate::time::{Date, TimeError};

/// Number of days since the common era (0001-01-01)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CeDays(i32);

impl CeDays {
  /// Instance with the maximum allowed value.
  pub const MAX: Self = Self(Date::MAX.ce_days());
  /// Instance with the minimum allowed value.
  pub const MIN: Self = Self(Date::MIN.ce_days());

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: i32) -> Result<Self, TimeError> {
    if num < Date::MIN.ce_days() || num > Date::MAX.ce_days() {
      return Err(TimeError::InvalidCeDays { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> i32 {
    self.0
  }
}
