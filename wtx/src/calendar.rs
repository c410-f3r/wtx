//! Simple time utilities

mod calendar_error;
mod instant;

#[cfg(feature = "calendar")]
#[macro_use]
mod macros;

#[cfg(feature = "calendar")]
mod ce_days;
#[cfg(feature = "calendar")]
mod date;
#[cfg(feature = "calendar")]
mod date_time;
#[cfg(feature = "calendar")]
mod day;
#[cfg(feature = "calendar")]
mod day_of_year;
#[cfg(feature = "calendar")]
mod duration;
#[cfg(feature = "calendar")]
mod format;
#[cfg(feature = "calendar")]
mod hour;
#[cfg(feature = "calendar")]
mod microsecond;
#[cfg(feature = "calendar")]
mod millisecond;
#[cfg(feature = "calendar")]
mod minute;
#[cfg(feature = "calendar")]
mod misc;
#[cfg(feature = "calendar")]
mod month;
#[cfg(feature = "calendar")]
mod nanosecond;
#[cfg(feature = "calendar")]
mod second;
#[cfg(feature = "calendar")]
mod time;
#[cfg(feature = "calendar")]
mod time_zone;
#[cfg(feature = "calendar")]
mod weekday;
#[cfg(feature = "calendar")]
mod year;

#[cfg(feature = "calendar")]
mod gated {
  pub use crate::calendar::{
    ce_days::CeDays,
    date::Date,
    date_time::DateTime,
    day::Day,
    day_of_year::DayOfYear,
    duration::Duration,
    format::{calendar_token::CalendarToken, parse_bytes_into_tokens},
    hour::Hour,
    microsecond::Microsecond,
    millisecond::Millisecond,
    minute::Minute,
    month::Month,
    nanosecond::Nanosecond,
    second::Second,
    time::Time,
    time_zone::*,
    weekday::Weekday,
    year::Year,
  };

  pub(crate) const DAYS_PER_4_YEARS: u16 = 1_461;
  pub(crate) const DAYS_PER_NON_LEAP_YEAR: u16 = 365;
  pub(crate) const DAYS_PER_QUADCENTURY: u32 = 146_097;
  /// Number of days between the Common Era and the UNIX epoch.
  pub(crate) const EPOCH_CE_DAYS: u32 = 719_163;
  pub(crate) const MINUTES_PER_HOUR: u8 = 60;
  pub(crate) const MILLISECONDS_PER_SECOND: u16 = 1_000;
  pub(crate) const NANOSECONDS_PER_MICROSECONDS: u32 = 1_000;
  pub(crate) const NANOSECONDS_PER_MILLISECOND: u32 = 1_000_000;
  pub(crate) const NANOSECONDS_PER_SECOND: u32 = 1_000_000_000;
  pub(crate) const SECONDS_PER_DAY: u32 = crate::calendar::misc::u16u32(SECONDS_PER_HOUR) * 24;
  pub(crate) const SECONDS_PER_HOUR: u16 = crate::calendar::misc::u8u16(SECONDS_PER_MINUTE) * 60;
  pub(crate) const SECONDS_PER_MINUTE: u8 = 60;
  pub(crate) const YEARS_PER_QUADCENTURY: u16 = 400;

  pub(crate) static DAYS_OF_MONTHS: [[u16; 12]; 2] = [
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
  ];
}

pub use calendar_error::CalendarError;
#[cfg(feature = "calendar")]
pub use gated::*;
pub use instant::Instant;
