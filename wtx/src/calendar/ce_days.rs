use crate::calendar::{CalendarError, Date};

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
  pub const fn from_num(num: i32) -> Result<Self, CalendarError> {
    if num < Date::MIN.ce_days() || num > Date::MAX.ce_days() {
      return Err(CalendarError::InvalidCeDays { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> i32 {
    self.0
  }
}

impl TryFrom<i32> for CeDays {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: i32) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
