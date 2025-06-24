mod format;
#[cfg(test)]
mod tests;

use crate::{
  calendar::{
    CalendarError, CalendarToken, Duration, Hour, MINUTES_PER_HOUR, Microsecond, Millisecond,
    Minute, NANOSECONDS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, Second,
    misc::{i32i64, u8i32, u8u32, u16i32, u16u32, u32i64},
    nanosecond::Nanosecond,
  },
  collection::ArrayString,
  de::u32_string,
};
use core::{
  fmt::{Debug, Display, Formatter},
  hint::unreachable_unchecked,
};

/// Clock time with nanosecond precision.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Time {
  hour: Hour,
  minute: Minute,
  second: Second,
  nanosecond: Nanosecond,
}

impl Time {
  /// Instance with the maximum allowed value of `23:59:59.999_999_999`
  pub const MAX: Self = Self::from_hms_ns(Hour::N23, Minute::N59, Second::N59, Nanosecond::MAX);
  /// Instance with the minimum allowed value of `00:00:00.000_000_000`
  pub const ZERO: Self = Self::from_hms_ns(Hour::N0, Minute::N0, Second::N0, Nanosecond::ZERO);

  /// New instance without nanosecond precision.
  #[inline]
  pub const fn from_hms(hour: Hour, minute: Minute, second: Second) -> Self {
    Self { hour, minute, second, nanosecond: Nanosecond::ZERO }
  }

  /// New instance with milliseconds precision.
  #[inline]
  pub const fn from_hms_ms(
    hour: Hour,
    minute: Minute,
    second: Second,
    millisecond: Millisecond,
  ) -> Self {
    Self::from_hms_ns(hour, minute, second, millisecond.to_ns())
  }

  /// New instance with nanosecond precision.
  #[inline]
  pub const fn from_hms_ns(
    hour: Hour,
    minute: Minute,
    second: Second,
    nanosecond: Nanosecond,
  ) -> Self {
    Self { hour, minute, second, nanosecond }
  }

  /// New instance with microseconds precision.
  #[inline]
  pub const fn from_hms_us(
    hour: Hour,
    minute: Minute,
    second: Second,
    microsecond: Microsecond,
  ) -> Self {
    Self::from_hms_ns(hour, minute, second, microsecond.to_ns())
  }

  /// Creates a new instance based on the string representation of the ISO-8601 specification.
  #[inline]
  pub fn from_iso_8601(bytes: &[u8]) -> crate::Result<Self> {
    static TOKENS: &[CalendarToken] = &[
      CalendarToken::TwoDigitHour,
      CalendarToken::Colon,
      CalendarToken::TwoDigitMinute,
      CalendarToken::Colon,
      CalendarToken::TwoDigitSecond,
      CalendarToken::DotNano,
    ];
    Self::parse(bytes, TOKENS.iter().copied())
  }

  /// Computes `self + duration`, returning an error if an overflow occurred.
  #[inline]
  pub const fn add(self, duration: Duration) -> Result<Self, CalendarError> {
    let (this, remaining) = self.overflowing_add(duration);
    if remaining > 0 {
      return Err(CalendarError::ArithmeticOverflow);
    }
    Ok(this)
  }

  /// Hours of a day
  #[inline]
  pub const fn hour(self) -> Hour {
    self.hour
  }

  /// ISO-8601 string representation
  #[inline]
  pub fn iso_8601(self) -> ArrayString<18> {
    let mut array = ArrayString::new();
    let _rslt0 = array.push_str(self.hour().num_str());
    let _rslt1 = array.push(':');
    let _rslt2 = array.push_str(self.minute().num_str());
    let _rslt3 = array.push(':');
    let _rslt4 = array.push_str(self.second().num_str());
    let nanosecond = self.nanosecond();
    if nanosecond.num() > 0 {
      let _rslt5 = array.push('.');
      let _rslt6 = array.push_str(&u32_string(nanosecond.num()));
    }
    array
  }

  /// Minutes of a hour.
  #[inline]
  pub const fn minute(self) -> Minute {
    self.minute
  }

  /// Nanosecond of a second
  #[inline]
  pub const fn nanosecond(self) -> Nanosecond {
    self.nanosecond
  }

  /// Adds the given `duration` to the current time, returning the number of *seconds*
  /// in the integral number of days ignored from the addition.
  #[must_use]
  pub const fn overflowing_add(self, duration: Duration) -> (Self, i64) {
    if duration.is_zero() {
      return (self, 0);
    }
    let mut seconds = u32i64(self.seconds_since_mn());
    let mut nanosecond = self.nanosecond.num().cast_signed();
    seconds = seconds.wrapping_add(duration.seconds());
    nanosecond = nanosecond.wrapping_add(duration.subsec_nanoseconds());
    manage_out_of_bounds!(@one, 0, NANOSECONDS_PER_SECOND.cast_signed(), nanosecond, seconds);
    let (day_seconds, this_hours, this_minutes, this_seconds) = Time::hms_from_seconds(seconds);
    (
      Self::from_hms_ns(
        match Hour::from_num(this_hours) {
          Ok(elem) => elem,
          // SAFETY: `hms_from_seconds` keeps `hours` within bounds
          Err(_) => unsafe { unreachable_unchecked() },
        },
        match Minute::from_num(this_minutes) {
          Ok(elem) => elem,
          // SAFETY: `hms_from_seconds` keeps `minutes` within bounds
          Err(_) => unsafe { unreachable_unchecked() },
        },
        match Second::from_num(this_seconds) {
          Ok(elem) => elem,
          // SAFETY: `hms_from_seconds` keeps `seconds` within bounds
          Err(_) => unsafe { unreachable_unchecked() },
        },
        match Nanosecond::from_num(nanosecond.cast_unsigned()) {
          Ok(elem) => elem,
          // SAFETY: `manage_out_of_bounds` keeps `nanosecond` within bounds
          Err(_) => unsafe { unreachable_unchecked() },
        },
      ),
      seconds.wrapping_sub(i32i64(day_seconds)),
    )
  }

  /// Subtracts the given `duration` from the current time, returning the number of *seconds*
  /// in the integral number of days ignored from the subtraction.
  #[must_use]
  pub const fn overflowing_sub(self, duration: Duration) -> (Self, i64) {
    let (time, rhs) = self.overflowing_add(duration.neg());
    (time, -rhs)
  }

  /// Seconds of a minute
  #[inline]
  pub const fn second(self) -> Second {
    self.second
  }

  /// The total number of seconds since midnight (00:00:00).
  #[inline]
  pub const fn seconds_since_mn(self) -> u32 {
    let mut rslt = (u8u32(self.hour().num())).wrapping_mul(u16u32(SECONDS_PER_HOUR));
    rslt = rslt.wrapping_add(u8u32(self.minute().num()).wrapping_mul(u8u32(SECONDS_PER_MINUTE)));
    rslt.wrapping_add(u8u32(self.second().num()))
  }

  /// Computes `self - duration`, returning an error if an underflow occurred.
  #[inline]
  pub const fn sub(self, duration: Duration) -> Result<Self, CalendarError> {
    let (this, remaining) = self.overflowing_sub(duration);
    if remaining < 0 {
      return Err(CalendarError::ArithmeticOverflow);
    }
    Ok(this)
  }

  /// Returns a new instance with the number of nanoseconds totally erased.
  #[inline]
  pub const fn trunc_to_sec(self) -> Self {
    let mut new = self;
    new.nanosecond = Nanosecond::ZERO;
    new
  }

  /// Returns a new instance with the number of nanoseconds truncated to microseconds.
  #[inline]
  pub const fn trunc_to_us(self) -> Self {
    let mut new = self;
    new.nanosecond = new.nanosecond.to_us().to_ns();
    new
  }

  pub(crate) const fn hms_from_seconds(seconds: i64) -> (i32, u8, u8, u8) {
    let day_seconds = seconds.rem_euclid(u32i64(SECONDS_PER_DAY)) as i32;
    let hour = (day_seconds / u16i32(SECONDS_PER_HOUR)) as u8;
    let minute = ((day_seconds % u16i32(SECONDS_PER_HOUR)) / u8i32(MINUTES_PER_HOUR)) as u8;
    let second = (day_seconds % u8i32(SECONDS_PER_MINUTE)) as u8;
    (day_seconds, hour, minute, second)
  }
}

impl Debug for Time {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso_8601())
  }
}

impl Default for Time {
  #[inline]
  fn default() -> Self {
    Self::ZERO
  }
}

impl Display for Time {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso_8601())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::calendar::Time;
  use core::fmt;
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
  };

  impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor;

      impl Visitor<'_> for LocalVisitor {
        type Value = Time;

        #[inline]
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          formatter.write_str("a formatted time string")
        }

        #[inline]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
          E: Error,
        {
          Time::from_iso_8601(value.as_bytes()).map_err(E::custom)
        }
      }

      deserializer.deserialize_str(LocalVisitor)
    }
  }

  impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(&self.iso_8601())
    }
  }
}
