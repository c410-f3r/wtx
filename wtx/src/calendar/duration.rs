use crate::calendar::{
  CalendarError, MILLISECONDS_PER_SECOND, NANOSECONDS_PER_MILLISECOND, NANOSECONDS_PER_SECOND,
  SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
  misc::{i32i64, u8i64, u16i64, u32i64},
};

/// A span of time with nanosecond precision.
///
/// Differently from [`core::time::Duration`], this structure allows negative durations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Duration {
  seconds: i64,
  nanosecond: i32,
}

impl Duration {
  /// Instance with the minimum allowed value.
  pub const MIN: Self =
    if let Ok(el) = Self::new(i64::MIN + 1, -999_999_999) { el } else { panic!() };
  /// Instance with the maximum allowed value.
  pub const MAX: Self = if let Ok(el) = Self::new(i64::MAX, 999_999_999) { el } else { panic!() };
  /// Instance without intervals.
  pub const ZERO: Self = if let Ok(el) = Self::new(0, 0) { el } else { panic!() };

  /// Creates a new instance from the specified number of days.
  #[inline]
  pub const fn from_days(days: i64) -> Result<Self, CalendarError> {
    let Some(seconds) = days.checked_mul(u32i64(SECONDS_PER_DAY)) else {
      return Err(CalendarError::ArithmeticOverflow);
    };
    Self::from_seconds(seconds)
  }

  /// Creates a new instance from the specified number of hours.
  #[inline]
  pub const fn from_hours(hours: i64) -> Result<Self, CalendarError> {
    let Some(seconds) = hours.checked_mul(u16i64(SECONDS_PER_HOUR)) else {
      return Err(CalendarError::ArithmeticOverflow);
    };
    Self::from_seconds(seconds)
  }

  /// Creates a new instance from the specified number of milliseconds.
  #[expect(clippy::arithmetic_side_effects, reason = "divisors ares constants")]
  #[expect(
    clippy::cast_possible_truncation,
    reason = "resulting values of divisions and modules don't extrapolate associated types"
  )]
  #[inline]
  pub const fn from_milliseconds(milliseconds: i64) -> Self {
    Self {
      seconds: milliseconds / u16i64(MILLISECONDS_PER_SECOND),
      nanosecond: {
        let rest = (milliseconds % u16i64(MILLISECONDS_PER_SECOND)) as i32;
        rest.wrapping_mul(NANOSECONDS_PER_MILLISECOND.cast_signed())
      },
    }
  }

  /// Creates a new instance from the specified number of minutes.
  #[inline]
  pub const fn from_minutes(minutes: i64) -> Result<Self, CalendarError> {
    let Some(seconds) = minutes.checked_mul(u8i64(SECONDS_PER_MINUTE)) else {
      return Err(CalendarError::ArithmeticOverflow);
    };
    Self::from_seconds(seconds)
  }

  /// Creates a new instance from the specified number of whole seconds.
  #[inline]
  pub const fn from_seconds(seconds: i64) -> Result<Duration, CalendarError> {
    if seconds == i64::MIN {
      return Err(CalendarError::ArithmeticOverflow);
    }
    Ok(Self { seconds, nanosecond: 0 })
  }

  /// Creates a new instance from the specified number of whole seconds and additional
  /// nanosecond.
  ///
  ///  If the number of nanosecond is greater than 1 billion (the number of nanosecond in a
  /// second), then it will carry over into the seconds provided.
  #[expect(clippy::arithmetic_side_effects, reason = "divisor is constant")]
  #[inline]
  pub const fn new(mut seconds: i64, mut nanosecond: i32) -> Result<Duration, CalendarError> {
    match seconds.checked_add(i32i64(nanosecond) / u32i64(NANOSECONDS_PER_SECOND)) {
      Some(elem) => {
        seconds = elem;
      }
      None => return Err(CalendarError::ArithmeticOverflow),
    }
    nanosecond %= NANOSECONDS_PER_SECOND.cast_signed();
    if seconds > 0 && nanosecond < 0 {
      seconds = seconds.wrapping_sub(1);
      nanosecond = nanosecond.wrapping_add(NANOSECONDS_PER_SECOND.cast_signed());
    } else if seconds < 0 && nanosecond > 0 {
      seconds = seconds.wrapping_add(1);
      nanosecond = nanosecond.wrapping_sub(NANOSECONDS_PER_SECOND.cast_signed());
    } else {
    }
    if seconds == i64::MIN {
      return Err(CalendarError::ArithmeticOverflow);
    }
    Ok(Self { seconds, nanosecond })
  }

  /// Returns the number of days contained in this instance.
  #[expect(clippy::arithmetic_side_effects, reason = "divisor is constant")]
  #[inline]
  pub const fn days(self) -> i64 {
    self.seconds() / u32i64(SECONDS_PER_DAY)
  }

  /// Returns the number of hours contained in this instance.
  #[expect(clippy::arithmetic_side_effects, reason = "divisor is constant")]
  #[inline]
  pub const fn hours(self) -> i64 {
    self.seconds / u16i64(SECONDS_PER_HOUR)
  }

  /// Returns `true` if the number of seconds and nanoseconds are zero
  #[inline]
  pub const fn is_zero(self) -> bool {
    self.seconds == 0 && self.nanosecond == 0
  }

  /// Returns the number of minutes contained in this instance.
  #[expect(clippy::arithmetic_side_effects, reason = "divisor is constant")]
  #[inline]
  pub const fn minutes(self) -> i64 {
    self.seconds / u8i64(SECONDS_PER_MINUTE)
  }

  /// Computes `-self`.
  #[expect(clippy::arithmetic_side_effects, reason = "constructors don't allow `i64::MAX` seconds")]
  #[inline]
  #[must_use]
  pub const fn neg(self) -> Self {
    Self { seconds: -self.seconds, nanosecond: -self.nanosecond }
  }

  /// Returns the number of _whole_ seconds contained in this instance.
  #[inline]
  pub const fn seconds(self) -> i64 {
    self.seconds
  }

  /// Returns the number of nanosecond past the number of whole seconds.
  #[inline]
  pub const fn subsec_nanoseconds(self) -> i32 {
    self.nanosecond
  }
}

impl TryFrom<core::time::Duration> for Duration {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: core::time::Duration) -> crate::Result<Self> {
    Ok(Self::new(from.as_secs().try_into()?, from.subsec_nanos().cast_signed())?)
  }
}

impl TryFrom<Duration> for core::time::Duration {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: Duration) -> crate::Result<Self> {
    Ok(Self::new(from.seconds.try_into()?, from.nanosecond.try_into()?))
  }
}
