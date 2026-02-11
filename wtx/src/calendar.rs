//! Simple time utilities

mod calendar_error;
mod instant;

#[macro_use]
mod macros;

mod ce_days;
mod date;
mod date_time;
mod day;
mod day_of_year;
mod duration;
mod format;
mod hour;
mod microsecond;
mod millisecond;
mod minute;
mod misc;
mod month;
mod nanosecond;
mod second;
mod time;
mod time_zone;
#[cfg(feature = "tracing-tree")]
mod tracing_tree_timer;
mod weekday;
mod year;

use crate::de::{U64String, u64_string};
pub use calendar_error::CalendarError;
pub use ce_days::CeDays;
pub use date::Date;
pub use date_time::DateTime;
pub use day::Day;
pub use day_of_year::DayOfYear;
pub use duration::Duration;
pub use format::{calendar_token::CalendarToken, parse_bytes_into_tokens};
pub use hour::Hour;
pub use instant::Instant;
pub use microsecond::Microsecond;
pub use millisecond::Millisecond;
pub use minute::Minute;
pub use month::Month;
pub use nanosecond::Nanosecond;
pub use second::Second;
pub use time::Time;
pub use time_zone::*;
#[cfg(feature = "tracing-tree")]
pub use tracing_tree_timer::TracingTreeTimer;
pub use weekday::Weekday;
pub use year::Year;

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
pub(crate) const SECONDS_PER_DAY: u32 = misc::u16u32(SECONDS_PER_HOUR) * 24;
pub(crate) const SECONDS_PER_HOUR: u16 = misc::u8u16(SECONDS_PER_MINUTE) * 60;
pub(crate) const SECONDS_PER_MINUTE: u8 = 60;
pub(crate) const YEARS_PER_QUADCENTURY: u16 = 400;

pub(crate) static DAYS_OF_MONTHS: [[u16; 12]; 2] = [
  [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
  [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
];

/// The current time in according to `cb` as a string.
#[inline]
pub fn timestamp_str(
  cb: impl FnOnce(core::time::Duration) -> u128,
) -> crate::Result<(u64, U64String)> {
  let number = Instant::now_timestamp(0).map(cb)?.try_into()?;
  Ok((number, u64_string(number)))
}
