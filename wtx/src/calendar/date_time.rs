mod format;
#[cfg(test)]
mod tests;

use crate::{
  calendar::{
    CalendarError, CeDays, Date, Duration, EPOCH_CE_DAYS, Hour, Minute, Nanosecond,
    SECONDS_PER_DAY, Second, Time, TimeToken,
    misc::{i32i64, u32i64},
  },
  collection::ArrayString,
};
use core::fmt::{Debug, Display, Formatter};

/// ISO-8601 representation without timezones.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime {
  date: Date,
  time: Time,
}

impl DateTime {
  /// Instance that refers the common era (0001-01-01).
  pub const CE: Self = Self::new(Date::CE, Time::ZERO);
  /// Instance that refers the UNIX epoch (1970-01-01).
  pub const EPOCH: Self = Self::new(Date::EPOCH, Time::ZERO);
  /// Instance with the maximum allowed value of `32768-12-31 24:59:59.999_999_999`
  pub const MAX: Self = Self::new(Date::MAX, Time::MAX);
  /// Instance with the minimum allowed value of `-32768-01-01 00:00:00.000_000_000`
  pub const MIN: Self = Self::new(Date::MIN, Time::ZERO);

  /// Creates a new instance based on the string representation of the ISO-8601 specification.
  #[inline]
  pub fn from_iso_8601(bytes: &[u8]) -> crate::Result<Self> {
    static TOKENS: &[TimeToken] = &[
      TimeToken::FourDigitYear,
      TimeToken::Dash,
      TimeToken::TwoDigitMonth,
      TimeToken::Dash,
      TimeToken::TwoDigitDay,
      TimeToken::Separator,
      TimeToken::TwoDigitHour,
      TimeToken::Colon,
      TimeToken::TwoDigitMinute,
      TimeToken::Colon,
      TimeToken::TwoDigitSecond,
      TimeToken::DotNano,
    ];
    Self::parse(bytes, TOKENS.iter().copied())
  }

  /// Creates a new instance from a UNIX timestamp expressed in seconds.
  #[inline]
  pub fn from_timestamp_secs(second: i64) -> crate::Result<Self> {
    Self::from_timestamp_secs_and_ns(second, Nanosecond::ZERO)
  }

  /// Creates a new instance from a UNIX timestamp expressed in seconds along side the number of
  /// nanoseconds.
  #[allow(
    clippy::arithmetic_side_effects,
    reason = "Divisions/modulos are using non-zero numbers but it can't see past a literal constant"
  )]
  #[inline]
  pub fn from_timestamp_secs_and_ns(seconds: i64, nanoseconds: Nanosecond) -> crate::Result<Self> {
    if seconds < Self::MIN.timestamp().0 || seconds > Self::MAX.timestamp().0 {
      return Err(CalendarError::InvalidTimestamp.into());
    }
    let days = seconds.div_euclid(SECONDS_PER_DAY.into()).wrapping_add(EPOCH_CE_DAYS.into());
    let (_, hour, minute, second) = Time::hms_from_seconds(seconds);
    Ok(Self::new(
      Date::from_ce_days(CeDays::from_num(days.try_into()?)?)?,
      Time::from_hms_ns(
        Hour::from_num(hour)?,
        Minute::from_num(minute)?,
        Second::from_num(second)?,
        nanoseconds,
      ),
    ))
  }

  /// New instance from basic parameters
  #[inline]
  pub const fn new(date: Date, time: Time) -> Self {
    Self { date, time }
  }

  /// Computes `self + duration`, returning an error if an overflow occurred.
  #[inline]
  pub const fn add(&self, duration: Duration) -> Result<Self, CalendarError> {
    let (time, remaining) = self.time.overflowing_add(duration);
    let rhs = match Duration::from_seconds(remaining) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    let date = match self.date.add(rhs) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, time))
  }

  /// See [`Date`].
  #[inline]
  pub const fn date(self) -> Date {
    self.date
  }

  /// ISO-8601 string representation
  #[inline]
  pub fn iso_8601(self) -> ArrayString<32> {
    let mut rslt = ArrayString::new();
    let _rslt0 = rslt.push_str(&self.date.iso_8601());
    let _rslt1 = rslt.push('T');
    let _rslt2 = rslt.push_str(&self.time.iso_8601());
    rslt
  }

  /// Computes `self - duration`, returning an error if an underflow occurred.
  #[inline]
  pub const fn sub(&self, duration: Duration) -> Result<Self, CalendarError> {
    let (time, remaining) = self.time.overflowing_sub(duration);
    let rhs = match Duration::from_seconds(remaining) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    let date = match self.date.sub(rhs) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, time))
  }

  /// See [`Time`].
  #[inline]
  pub const fn time(self) -> Time {
    self.time
  }

  /// UNIX timestamp in seconds as well as the number of nanoseconds.
  #[inline]
  pub const fn timestamp(self) -> (i64, Nanosecond) {
    let mut rslt = i32i64(self.date.ce_days());
    rslt = rslt.wrapping_sub(u32i64(EPOCH_CE_DAYS));
    rslt = rslt.wrapping_mul(u32i64(SECONDS_PER_DAY));
    (rslt.wrapping_add(u32i64(self.time.seconds_since_mn())), self.time.nanosecond())
  }
}

impl Debug for DateTime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso_8601())
  }
}

impl Default for DateTime {
  #[inline]
  fn default() -> Self {
    Self::EPOCH
  }
}

impl Display for DateTime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso_8601())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::calendar::DateTime;
  use core::fmt;
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
  };

  impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor;

      impl Visitor<'_> for LocalVisitor {
        type Value = DateTime;

        #[inline]
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          formatter.write_str("a formatted date and time string")
        }

        #[inline]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
          E: Error,
        {
          DateTime::from_iso_8601(value.as_bytes()).map_err(E::custom)
        }
      }

      deserializer.deserialize_str(LocalVisitor)
    }
  }

  impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(&self.iso_8601())
    }
  }
}
