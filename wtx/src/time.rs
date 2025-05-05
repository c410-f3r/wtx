//! Simple UTC-only time utilities.

mod ce_days;
mod date;
mod date_time;
mod day;
mod day_of_year;
mod hour;
mod instant;
mod microsecond;
mod millisecond;
mod minute;
mod misc;
mod month;
mod nanosecond;
mod second;
#[allow(clippy::module_inception, reason = "there isn't a better name")]
mod time;
mod time_error;
mod year;

pub use ce_days::CeDays;
pub use date::Date;
pub use date_time::DateTime;
pub use day::Day;
pub use day_of_year::DayOfYear;
pub use hour::Hour;
pub use instant::Instant;
pub use microsecond::Microsecond;
pub use millisecond::Millisecond;
pub use minute::Minute;
pub use month::Month;
pub use nanosecond::Nanosecond;
pub use second::Second;
pub use time::Time;
pub use time_error::TimeError;
pub use year::Year;

pub(crate) const DAYS_PER_4_YEARS: u16 = 1_461;
pub(crate) const DAYS_PER_400_YEARS_I32: i32 = 146_097;
pub(crate) const DAYS_PER_NON_LEAP_YEAR_I16: i16 = 365;
pub(crate) const DAYS_PER_NON_LEAP_YEAR_U16: u16 = 365;
pub(crate) const MINUTES_PER_HOUR: u8 = 60;
pub(crate) const NANOSECONDS_PER_MICROSECONDS: u32 = 1_000;
pub(crate) const NANOSECONDS_PER_MILLISECOND: u32 = 1_000_000;
pub(crate) const SECONDS_PER_DAY: u32 = misc::u16u32(SECONDS_PER_HOUR) * 24;
pub(crate) const SECONDS_PER_HOUR: u16 = misc::u8u16(SECONDS_PER_MINUTE) * 60;
pub(crate) const SECONDS_PER_MINUTE: u8 = 60;
/// Number of days between the UNIX epoch and -0001-12-31.
pub(crate) const UNIX_EPOCH_DAYS: u32 = 719_163;

static DAYS_OF_MONTHS: [[u16; 12]; 2] = [
  [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
  [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
];
