mod format;

use crate::{
  collection::ArrayString,
  time::{
    CeDays, Date, Hour, MINUTES_PER_HOUR, Minute, Nanosecond, SECONDS_PER_DAY, SECONDS_PER_HOUR,
    SECONDS_PER_MINUTE, Second, Time, TimeError, UNIX_EPOCH_DAYS,
    misc::{i32i64, u32i64},
  },
};
use core::fmt::{Debug, Display, Formatter};

type DateTimeString = ArrayString<32>;

/// ISO-8601 representation with a fixed UTC timezone.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime {
  date: Date,
  time: Time,
}

impl DateTime {
  /// Instance that refers the common era (0001-01-01).
  pub const CE: Self = Self::new(Date::CE, Time::MIN);
  /// Instance that refers the UNIX epoch (1970-01-01).
  pub const EPOCH: Self = Self::new(Date::EPOCH, Time::MIN);
  /// Instance with the maximum allowed value of `32768-12-31 24:59:59.999_999_999`
  pub const MAX: Self = Self::new(Date::MAX, Time::MAX);
  /// Instance with the minimum allowed value of `-32768-01-01 00:00:00.000_000_000`
  pub const MIN: Self = Self::new(Date::MIN, Time::MIN);

  /// Creates a new instance from a UNIX timestamp expressed in seconds.
  #[inline]
  pub fn from_timestamp_secs(timestamp: i64) -> crate::Result<Self> {
    Self::from_timestamp_secs_and_ns(timestamp, Nanosecond::ZERO)
  }

  /// Creates a new instance from a UNIX timestamp expressed in seconds along side the number of
  /// nanoseconds
  #[allow(
    clippy::arithmetic_side_effects,
    reason = "Divisions/modulos are using non-zero numbers but it can't see past a literal constant"
  )]
  #[inline]
  pub fn from_timestamp_secs_and_ns(timestamp: i64, nanosecond: Nanosecond) -> crate::Result<Self> {
    if timestamp < Self::MIN.timestamp().0 || timestamp > Self::MAX.timestamp().0 {
      return Err(TimeError::InvalidTimestamp.into());
    }
    let days = timestamp.div_euclid(SECONDS_PER_DAY.into()).wrapping_add(UNIX_EPOCH_DAYS.into());
    let secs = timestamp.rem_euclid(SECONDS_PER_DAY.into());
    let hour = (secs / i64::from(SECONDS_PER_HOUR)).try_into()?;
    let minute = ((secs % i64::from(SECONDS_PER_HOUR)) / i64::from(MINUTES_PER_HOUR)).try_into()?;
    let second = (secs % i64::from(SECONDS_PER_MINUTE)).try_into()?;
    Ok(Self::new(
      Date::from_ce_days(CeDays::from_num(days.try_into()?)?)?,
      Time::from_hms_ns(
        Hour::from_num(hour)?,
        Minute::from_num(minute)?,
        Second::from_num(second)?,
        nanosecond,
      ),
    ))
  }

  /// New instance from basic parameters
  #[inline]
  pub const fn new(date: Date, time: Time) -> Self {
    Self { date, time }
  }

  /// See [`Date`].
  #[inline]
  pub const fn date(self) -> Date {
    self.date
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
    rslt = rslt.wrapping_sub(u32i64(UNIX_EPOCH_DAYS));
    rslt = rslt.wrapping_mul(u32i64(SECONDS_PER_DAY));
    (rslt.wrapping_add(u32i64(self.time.seconds_from_mn())), self.time.nanosecond())
  }

  /// String representation
  #[inline]
  pub fn to_str(self) -> DateTimeString {
    let mut rslt = DateTimeString::new();
    let _rslt0 = rslt.push_str(&self.date.to_str());
    let _rslt1 = rslt.push('T');
    let _rslt2 = rslt.push_str(&self.time.to_str());
    let _rslt3 = rslt.push('Z');
    rslt
  }
}

impl Debug for DateTime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.to_str())
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
    f.write_str(&self.to_str())
  }
}

#[cfg(test)]
mod tests {
  use crate::time::{Date, DateTime, DayOfYear, Hour, Minute, Nanosecond, Second, Time, Year};

  fn _2025_04_20_14_20_30_1234() -> DateTime {
    DateTime::new(
      Date::new(Year::from_num(2025).unwrap(), DayOfYear::from_num(110).unwrap()).unwrap(),
      Time::from_hms_ns(Hour::N14, Minute::N20, Second::N30, Nanosecond::from_num(1234).unwrap()),
    )
  }

  #[test]
  fn from_timestamp_secs() {
    let elements = [
      (1662921288, "2022-09-11T18:34:48Z"),
      (1662921287, "2022-09-11T18:34:47Z"),
      (-2208936075, "1900-01-01T14:38:45Z"),
      (-5337182663, "1800-11-15T01:15:37Z"),
      (0000000000, "1970-01-01T00:00:00Z"),
      (119731017, "1973-10-17T18:36:57Z"),
      (1234567890, "2009-02-13T23:31:30Z"),
      (2034061609, "2034-06-16T09:06:49Z"),
    ];
    for (timestamp, str) in elements {
      let instance = DateTime::from_timestamp_secs(timestamp).unwrap();
      assert_eq!(instance.to_str().as_str(), str);
      assert_eq!(instance.timestamp().0, timestamp);
    }
  }

  #[test]
  fn timestamp() {
    assert_eq!(DateTime::MIN.timestamp().0, -1096193779200);
    assert_eq!(DateTime::MAX.timestamp().0, 971859427199);
    assert_eq!(_2025_04_20_14_20_30_1234().timestamp().0, 1745158830);
  }

  #[test]
  fn to_str() {
    assert_eq!(DateTime::MIN.to_str().as_str(), "-32767-01-01T00:00:00Z");
    assert_eq!(DateTime::MAX.to_str().as_str(), "32766-12-31T23:59:59.999999999Z");
    assert_eq!(_2025_04_20_14_20_30_1234().to_str().as_str(), "2025-04-20T14:20:30.1234Z");
  }
}
