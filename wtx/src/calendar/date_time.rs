mod format;
#[cfg(test)]
mod tests;

use crate::{
  calendar::{
    CalendarError, CalendarToken, CeDays, Date, Duration, EPOCH_CE_DAYS, Hour, Minute, Nanosecond,
    SECONDS_PER_DAY, Second, Time, TimeZone, Utc,
    misc::{i32i64, u32i64},
  },
  collection::{ArrayString, ArrayStringU8},
};
use core::fmt::{Debug, Display, Formatter};

/// ISO-8601 representation with timezones.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime<TZ> {
  date: Date,
  time: Time,
  tz: TZ,
}

impl DateTime<Utc> {
  /// Instance that refers the common era (0001-01-01).
  pub const CE: Self = Self::new(Date::CE, Time::ZERO, Utc);
  /// Instance that refers the UNIX epoch (1970-01-01).
  pub const EPOCH: Self = Self::new(Date::EPOCH, Time::ZERO, Utc);
  /// Instance with the maximum allowed value of `32768-12-31 24:59:59.999_999_999`
  pub const MAX: Self = Self::new(Date::MAX, Time::MAX, Utc);
  /// Instance with the minimum allowed value of `-32768-01-01 00:00:00.000_000_000`
  pub const MIN: Self = Self::new(Date::MIN, Time::ZERO, Utc);

  /// Creates a new instance from a UNIX timestamp expressed in seconds.
  #[inline]
  pub fn from_timestamp_secs(second: i64) -> crate::Result<Self> {
    Self::from_timestamp_secs_and_ns(second, Nanosecond::ZERO)
  }

  /// Creates a new instance from a UNIX timestamp expressed in seconds along side the number of
  /// nanoseconds.
  #[inline]
  pub fn from_timestamp_secs_and_ns(seconds: i64, nanoseconds: Nanosecond) -> crate::Result<Self> {
    if seconds < Self::MIN.timestamp_secs_and_ns().0
      || seconds > Self::MAX.timestamp_secs_and_ns().0
    {
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
      Utc,
    ))
  }
}

impl<TZ> DateTime<TZ>
where
  TZ: TimeZone,
{
  /// Creates a new instance based on the string representation of the ISO-8601 specification.
  #[inline]
  pub fn from_iso8601(bytes: &[u8]) -> crate::Result<Self> {
    static TOKENS: &[CalendarToken] = &[
      CalendarToken::FourDigitYear,
      CalendarToken::Dash,
      CalendarToken::TwoDigitMonth,
      CalendarToken::Dash,
      CalendarToken::TwoDigitDay,
      CalendarToken::Separator,
      CalendarToken::TwoDigitHour,
      CalendarToken::Colon,
      CalendarToken::TwoDigitMinute,
      CalendarToken::Colon,
      CalendarToken::TwoDigitSecond,
      CalendarToken::DotNano,
      CalendarToken::TimeZone,
    ];
    Self::parse(bytes, TOKENS.iter().copied())
  }

  /// New instance from basic parameters
  #[inline]
  pub const fn new(date: Date, time: Time, time_zone: TZ) -> Self {
    Self { date, time, tz: time_zone }
  }

  /// Computes `self + duration`, returning an error if an overflow occurred.
  #[inline]
  pub const fn add(self, duration: Duration) -> Result<Self, CalendarError> {
    if duration.is_zero() {
      return Ok(self);
    }
    let (time, remaining) = self.time.overflowing_add(duration);
    let rhs = match Duration::from_seconds(remaining) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    let date = match self.date.add(rhs) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, time, self.tz))
  }

  /// See [`Date`].
  #[inline]
  pub const fn date(self) -> Date {
    self.date
  }

  /// ISO-8601 string representation
  #[inline]
  pub fn iso8601(self) -> ArrayStringU8<38> {
    let mut rslt = ArrayString::new();
    let _rslt0 = rslt.push_str(&self.date.iso8601());
    let _rslt1 = rslt.push('T');
    let _rslt2 = rslt.push_str(&self.time.iso8601());
    let _rslt3 = rslt.push_str(&self.tz.iso8601());
    rslt
  }

  /// Computes `self - duration`, returning an error if an underflow occurred.
  #[inline]
  pub const fn sub(self, duration: Duration) -> Result<Self, CalendarError> {
    let (time, remaining) = self.time.overflowing_sub(duration);
    let rhs = match Duration::from_seconds(remaining) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    let date = match self.date.sub(rhs) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, time, self.tz))
  }

  /// See [`Time`].
  #[inline]
  pub const fn time(self) -> Time {
    self.time
  }

  /// UNIX timestamp in seconds as well as the number of nanoseconds.
  ///
  /// It is worth noting that it is much cheaper to get the timestamp using `Instant`.
  #[inline]
  pub const fn timestamp_secs_and_ns(self) -> (i64, Nanosecond) {
    let mut rslt = i32i64(self.date.ce_days());
    rslt = rslt.wrapping_sub(u32i64(EPOCH_CE_DAYS));
    rslt = rslt.wrapping_mul(u32i64(SECONDS_PER_DAY));
    (rslt.wrapping_add(u32i64(self.time.seconds_since_mn())), self.time.nanosecond())
  }

  /// Returns a new instance with the internal values converted to the provided timezone.
  #[inline]
  pub fn to_tz<NTZ>(self, tz: NTZ) -> Result<DateTime<NTZ>, CalendarError>
  where
    NTZ: TimeZone,
  {
    if (TZ::IS_LOCAL || TZ::IS_UTC) && (NTZ::IS_LOCAL || NTZ::IS_UTC) {
      return Ok(DateTime::new(self.date, self.time, tz));
    }
    let date_time = self.to_utc()?.add(Duration::from_minutes(i64::from(tz.minutes()))?)?;
    Ok(DateTime::new(date_time.date, date_time.time, tz))
  }

  /// Returns a new instance with the internal values converted to UTC.
  #[inline]
  pub fn to_utc(self) -> Result<DateTime<Utc>, CalendarError> {
    if TZ::IS_LOCAL || TZ::IS_UTC {
      Ok(DateTime::new(self.date, self.time, Utc))
    } else {
      let date_time = self.sub(Duration::from_minutes(i64::from(self.tz.minutes()))?)?;
      Ok(DateTime::new(date_time.date, date_time.time, Utc))
    }
  }

  /// Returns a new instance with the number of nanoseconds truncated to milliseconds.
  #[inline]
  #[must_use]
  pub const fn trunc_to_ms(self) -> Self {
    let mut new = self;
    new.time = new.time.trunc_to_ms();
    new
  }

  /// Returns a new instance with the number of nanoseconds totally erased.
  #[inline]
  #[must_use]
  pub const fn trunc_to_sec(self) -> Self {
    let mut new = self;
    new.time = new.time.trunc_to_sec();
    new
  }

  /// Returns a new instance with the number of nanoseconds truncated to microseconds.
  #[inline]
  #[must_use]
  pub const fn trunc_to_us(self) -> Self {
    let mut new = self;
    new.time = new.time.trunc_to_us();
    new
  }
}

impl<TZ> Debug for DateTime<TZ>
where
  TZ: TimeZone,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso8601())
  }
}

impl Default for DateTime<Utc> {
  #[inline]
  fn default() -> Self {
    Self::EPOCH
  }
}

impl<TZ> Display for DateTime<TZ>
where
  TZ: TimeZone,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso8601())
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::calendar::{DateTime, TimeZone};
  use core::{fmt, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
  };

  impl<'de, TZ> Deserialize<'de> for DateTime<TZ>
  where
    TZ: TimeZone,
  {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor<TZ>(PhantomData<TZ>);

      impl<TZ> Visitor<'_> for LocalVisitor<TZ>
      where
        TZ: TimeZone,
      {
        type Value = DateTime<TZ>;

        #[inline]
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          formatter.write_str("a formatted date and time string")
        }

        #[inline]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
          E: Error,
        {
          DateTime::from_iso8601(value.as_bytes()).map_err(E::custom)
        }
      }

      deserializer.deserialize_str(LocalVisitor(PhantomData))
    }
  }

  impl<TZ> Serialize for DateTime<TZ>
  where
    TZ: TimeZone,
  {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(&self.iso8601())
    }
  }
}
