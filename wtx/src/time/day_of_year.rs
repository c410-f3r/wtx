use crate::time::TimeError;

/// This particular structure can represent at most one second in nanoseconds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DayOfYear(u16);

impl DayOfYear {
  /// Instance with the minimum allowed value of `1`
  pub const ONE: Self = Self(1);
  /// Instance with the `365` value.
  pub const N365: Self = Self(365);
  /// Instance with the maximum allowed value of `366`
  pub const N366: Self = Self(366);

  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u16) -> Result<Self, TimeError> {
    if num < 1 || num > 366 {
      return Err(TimeError::InvalidDayOfTheYear { received: num });
    }
    Ok(Self(num))
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u16 {
    self.0
  }
}

impl TryFrom<u16> for DayOfYear {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u16) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
