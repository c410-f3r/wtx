mod format;
#[cfg(test)]
mod tests;

use crate::{
  calendar::{
    CalendarError, CalendarToken, CeDays, Date, DayOfYear, EPOCH_CE_DAYS, Hour, IDuration, Instant,
    Nanosecond, SECONDS_PER_DAY, SECONDS_PER_MINUTE, Sixty, Time, TimeZone, Utc, Year,
  },
  collections::{ArrayStr, ArrayStrU8},
  misc::int_conv::{i16i64, i32i64, u8i64, u8u64, u16u64, u32i64, u32u64},
};
use core::{
  fmt::{Debug, Display, Formatter},
  hint::unreachable_unchecked,
};

/// ISO-8601 representation with timezones.
//
// | xxxxxxxxxxxxxxxx | xxxxxxxxx       | xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx | xxxxxx | xxxxxx | xxxxx |
// | year             | day of the year | nanosecond                     | second | minute | hour  |
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Datetime<TZ> {
  data: [u8; 9],
  tz: TZ,
}

impl Datetime<Utc> {
  /// Instance that refers the common era (0001-01-01).
  pub const CE: Self = Self::new(Date::CE, Time::ZERO, Utc);
  /// Instance that refers the UNIX epoch (1970-01-01).
  pub const EPOCH: Self = Self::new(Date::EPOCH, Time::ZERO, Utc);
  /// Instance with the maximum allowed value of `32767-12-31 23:59:59.999_999_999`
  pub const MAX: Self = Self::new(Date::MAX, Time::MAX, Utc);
  /// Instance with the minimum allowed value of `-32767-01-01 00:00:00.000_000_000`
  pub const MIN: Self = Self::new(Date::MIN, Time::ZERO, Utc);

  /// Creates a new instance from a UNIX timestamp expressed in seconds.
  #[inline]
  pub const fn from_timestamp_secs(second: i64) -> Result<Self, CalendarError> {
    Self::from_timestamp_secs_and_ns(second, Nanosecond::ZERO)
  }

  /// Creates a new instance from a UNIX timestamp expressed in seconds alongside the number of
  /// nanoseconds.
  #[expect(
    clippy::cast_possible_truncation,
    reason = "resulting values of divisions and modules don't extrapolate associated types"
  )]
  #[inline]
  pub const fn from_timestamp_secs_and_ns(
    seconds: i64,
    nanosecond: Nanosecond,
  ) -> Result<Self, CalendarError> {
    if seconds < Self::MIN.timestamp_secs_and_ns_utc().0
      || seconds > Self::MAX.timestamp_secs_and_ns_utc().0
    {
      return Err(CalendarError::InvalidTimestamp);
    }
    let days = seconds.div_euclid(u32i64(SECONDS_PER_DAY)).wrapping_add(u32i64(EPOCH_CE_DAYS));
    let (_, hour, minute, second) = Time::hms_from_seconds(seconds);
    let ce_days = match CeDays::from_num(days as i32) {
      Ok(val) => val,
      Err(err) => return Err(err),
    };
    let date = match Date::from_ce_days(ce_days) {
      Ok(val) => val,
      Err(err) => return Err(err),
    };
    let local_hour = match Hour::from_num(hour) {
      Ok(val) => val,
      Err(err) => return Err(err),
    };
    let local_minute = match Sixty::from_num(minute) {
      Ok(val) => val,
      Err(err) => return Err(err),
    };
    let local_second = match Sixty::from_num(second) {
      Ok(val) => val,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, Time::from_hms_ns(local_hour, local_minute, local_second, nanosecond), Utc))
  }

  /// Shortcut for [`Datetime::now`].
  #[inline]
  pub fn now() -> crate::Result<Self> {
    Instant::now_datetime()
  }

  /// A [`Self::now`] version where the number of seconds are erased.
  #[inline]
  pub fn now_trunc_to_secs() -> crate::Result<Self> {
    Ok(Datetime::now()?.trunc_to_secs())
  }

  // FIXME(STABLE): Use `timestamp_secs_and_ns` with constant traits
  #[inline]
  const fn timestamp_secs_and_ns_utc(self) -> (i64, Nanosecond) {
    let time = self.time();
    let mut rslt = i32i64(self.date().ce_days());
    rslt = rslt.wrapping_sub(u32i64(EPOCH_CE_DAYS));
    rslt = rslt.wrapping_mul(u32i64(SECONDS_PER_DAY));
    rslt = rslt.wrapping_add(u32i64(time.seconds_since_mn()));
    (rslt, time.nanosecond())
  }
}

impl<TZ> Datetime<TZ>
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
    Self { data: Self::pack(date, time), tz: time_zone }
  }

  /// Computes `self + duration`, returning an error if an overflow occurred.
  #[inline]
  pub const fn add(self, duration: IDuration) -> Result<Self, CalendarError> {
    if duration.is_zero() {
      return Ok(self);
    }
    let (time, remaining) = self.time().overflowing_add(duration);
    let rhs = match IDuration::from_seconds(remaining) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    let date = match self.date().add(rhs) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, time, self.tz))
  }

  /// See [`Date`].
  #[inline]
  pub const fn date(self) -> Date {
    let year_num = u16::from_be_bytes([self.data[0], self.data[1]]) ^ 0b1000_0000_0000_0000;
    let rest_num = Self::rest_num(self.data);
    let Ok(year) = Year::from_num(year_num.cast_signed()) else {
      // SAFETY: `data` only stores instances created by `Year::from_num`
      unsafe { unreachable_unchecked() }
    };
    let Ok(day_of_year) = DayOfYear::from_num(Self::day_of_year_bits(rest_num)) else {
      // SAFETY: `data` only stores instances created by `DayOfYear::from_num`
      unsafe { unreachable_unchecked() }
    };
    let Ok(elem) = Date::new(year, day_of_year) else {
      // SAFETY: `data` only stores instances created by `Date::new`
      unsafe { unreachable_unchecked() }
    };
    elem
  }

  /// ISO-8601 string representation
  #[inline]
  pub fn iso8601(self, tz: bool) -> ArrayStrU8<38> {
    let mut rslt = ArrayStr::new();
    let _rslt0 = rslt.push_str(&self.date().iso8601());
    let _rslt1 = rslt.push('T');
    let _rslt2 = rslt.push_str(&self.time().iso8601());
    if tz {
      let _rslt3 = rslt.push_str(&self.tz.iso8601());
    }
    rslt
  }

  /// Computes `self - duration`, returning an error if an underflow occurred.
  #[inline]
  pub const fn sub(self, duration: IDuration) -> Result<Self, CalendarError> {
    let (time, remaining) = self.time().overflowing_sub(duration);
    let rhs = match IDuration::from_seconds(remaining) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    let date = match self.date().sub(rhs) {
      Ok(elem) => elem,
      Err(err) => return Err(err),
    };
    Ok(Self::new(date, time, self.tz))
  }

  /// See [`Time`].
  #[inline]
  pub const fn time(self) -> Time {
    let rest_num = Self::rest_num(self.data);
    let Ok(hour) = Hour::from_num(Self::hour_bits(rest_num)) else {
      // SAFETY: `data` only stores instances created by `Hour::from_num`
      unsafe { unreachable_unchecked() }
    };
    let Ok(minute) = Sixty::from_num(Self::minute_bits(rest_num)) else {
      // SAFETY: `data` only stores instances created by `Sixty::from_num`
      unsafe { unreachable_unchecked() }
    };
    let Ok(second) = Sixty::from_num(Self::second_bits(rest_num)) else {
      // SAFETY: `data` only stores instances created by `Sixty::from_num`
      unsafe { unreachable_unchecked() }
    };
    let Ok(nanosecond) = Nanosecond::from_num(Self::nanosecond_bits(rest_num)) else {
      // SAFETY: `data` only stores instances created by `Nanosecond::from_num`
      unsafe { unreachable_unchecked() }
    };
    Time::from_hms_ns(hour, minute, second, nanosecond)
  }

  /// UNIX timestamp in seconds as well as the number of nanoseconds.
  #[inline]
  pub fn timestamp_secs_and_ns(self) -> (i64, Nanosecond) {
    let time = self.time();
    let mut rslt = i32i64(self.date().ce_days());
    rslt = rslt.wrapping_sub(u32i64(EPOCH_CE_DAYS));
    rslt = rslt.wrapping_mul(u32i64(SECONDS_PER_DAY));
    rslt = rslt.wrapping_add(u32i64(time.seconds_since_mn()));
    rslt = rslt.wrapping_sub(i16i64(self.tz.minutes()).wrapping_mul(u8i64(SECONDS_PER_MINUTE)));
    (rslt, time.nanosecond())
  }

  /// Returns a new instance with the internal values converted to the provided timezone.
  #[inline]
  pub fn to_tz<NTZ>(self, tz: NTZ) -> Result<Datetime<NTZ>, CalendarError>
  where
    NTZ: TimeZone,
  {
    if (TZ::IS_LOCAL || TZ::IS_UTC) && (NTZ::IS_LOCAL || NTZ::IS_UTC) {
      return Ok(Datetime::new(self.date(), self.time(), tz));
    }
    let datetime = self.to_utc()?.add(IDuration::from_minutes(i64::from(tz.minutes()))?)?;
    Ok(Datetime::new(datetime.date(), datetime.time(), tz))
  }

  /// Returns a new instance with the internal values converted to UTC.
  #[inline]
  pub fn to_utc(self) -> Result<Datetime<Utc>, CalendarError> {
    if TZ::IS_LOCAL || TZ::IS_UTC {
      Ok(Datetime::new(self.date(), self.time(), Utc))
    } else {
      let datetime = self.sub(IDuration::from_minutes(i64::from(self.tz.minutes()))?)?;
      Ok(Datetime::new(datetime.date(), datetime.time(), Utc))
    }
  }

  /// Returns a new instance with the number of nanoseconds truncated to milliseconds.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn trunc_to_ms(self) -> Self {
    Self::new(self.date(), self.time().trunc_to_ms(), self.tz)
  }

  /// Returns a new instance with the number of nanoseconds totally erased.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn trunc_to_secs(self) -> Self {
    Self::new(self.date(), self.time().trunc_to_sec(), self.tz)
  }

  /// Returns a new instance with the number of nanoseconds truncated to microseconds.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn trunc_to_us(self) -> Self {
    Self::new(self.date(), self.time().trunc_to_us(), self.tz)
  }

  #[inline]
  const fn day_of_year_bits(rest_num: u64) -> u16 {
    ((rest_num >> 47) & 0b1_1111_1111) as u16
  }

  #[inline]
  const fn hour_bits(rest_num: u64) -> u8 {
    ((rest_num >> 42) & 0b1_1111) as u8
  }

  #[inline]
  const fn minute_bits(rest_num: u64) -> u8 {
    ((rest_num >> 36) & 0b11_1111) as u8
  }

  #[inline]
  const fn nanosecond_bits(rest_num: u64) -> u32 {
    (rest_num & 0b11_1111_1111_1111_1111_1111_1111_1111) as u32
  }

  #[inline]
  const fn pack(date: Date, time: Time) -> [u8; 9] {
    let year = (date.year().num().cast_unsigned() ^ 0b1000_0000_0000_0000).to_be_bytes();
    let rest_num = (u16u64(date.day_of_year().num()) << 47)
      | (u8u64(time.hour().num()) << 42)
      | (u8u64(time.minute().num()) << 36)
      | (u8u64(time.second().num()) << 30)
      | u32u64(time.nanosecond().num());
    let rest = rest_num.to_be_bytes();
    [year[0], year[1], rest[1], rest[2], rest[3], rest[4], rest[5], rest[6], rest[7]]
  }

  #[inline]
  const fn rest_num(data: [u8; 9]) -> u64 {
    u64::from_be_bytes([0, data[2], data[3], data[4], data[5], data[6], data[7], data[8]])
  }

  #[inline]
  const fn second_bits(rest_num: u64) -> u8 {
    ((rest_num >> 30) & 0b11_1111) as u8
  }
}

impl<TZ> Debug for Datetime<TZ>
where
  TZ: TimeZone,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso8601(true))
  }
}

impl Default for Datetime<Utc> {
  #[inline]
  fn default() -> Self {
    Self::EPOCH
  }
}

impl<TZ> Display for Datetime<TZ>
where
  TZ: TimeZone,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str(&self.iso8601(true))
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::calendar::{Datetime, TimeZone};
  use core::{fmt, marker::PhantomData};
  use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
  };

  impl<'de, TZ> Deserialize<'de> for Datetime<TZ>
  where
    TZ: TimeZone,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct LocalVisitor<TZ>(PhantomData<TZ>);

      impl<TZ> Visitor<'_> for LocalVisitor<TZ>
      where
        TZ: TimeZone,
      {
        type Value = Datetime<TZ>;

        #[inline]
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          formatter.write_str("a formatted date and time string")
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
          E: Error,
        {
          Datetime::from_iso8601(v).map_err(E::custom)
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
          E: Error,
        {
          Datetime::from_iso8601(v.as_bytes()).map_err(E::custom)
        }
      }

      deserializer.deserialize_str(LocalVisitor(PhantomData))
    }
  }

  impl<TZ> Serialize for Datetime<TZ>
  where
    TZ: TimeZone,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(&self.iso8601(true))
    }
  }
}
