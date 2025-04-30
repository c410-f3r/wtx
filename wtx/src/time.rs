//! Simple UTC-only time utilities.

mod ce_days;
mod date;
mod date_time;
mod day;
mod day_of_year;
mod hour;
mod instant;
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
pub use minute::Minute;
pub use month::Month;
pub use nanosecond::Nanosecond;
pub use second::Second;
pub use time::Time;
pub use time_error::TimeError;
pub use year::Year;

const DAYS_PER_4_YEARS: u16 = 1_461;
const DAYS_PER_400_YEARS_I32: i32 = 146_097;
const DAYS_PER_NON_LEAP_YEAR_I16: i16 = 365;
const DAYS_PER_NON_LEAP_YEAR_U16: u16 = 365;
const MINUTES_PER_HOUR: u8 = 60;
const NANOSECONDS_PER_MILLISECOND: u32 = 1_000_000;
const SECONDS_PER_DAY: u32 = misc::u16u32(SECONDS_PER_HOUR) * 24;
const SECONDS_PER_HOUR: u16 = misc::u8u16(SECONDS_PER_MINUTE) * 60;
const SECONDS_PER_MINUTE: u8 = 60;
/// Number of days between the UNIX epoch and -0001-12-31.
const UNIX_EPOCH_DAYS: u32 = 719_163;
