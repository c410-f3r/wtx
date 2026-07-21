//! Simple time utilities

#![expect(clippy::as_conversions, reason = "lack of const trait")]

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
#[cfg(feature = "epoch-sync")]
mod epoch_offset;
mod format;
mod hour;
mod microsecond;
mod millisecond;
mod misc;
mod month;
mod nanosecond;
mod sixty;
mod time;
mod time_zone;
#[cfg(feature = "tracing-tree")]
mod tracing_tree_timer;
mod weekday;
mod year;

use crate::codec::{U64String, u64_string};
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
pub use month::Month;
pub use nanosecond::Nanosecond;
pub use sixty::Sixty;
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
pub(crate) const SECONDS_PER_DAY: u32 = crate::misc::int_conv::u16u32(SECONDS_PER_HOUR) * 24;
pub(crate) const SECONDS_PER_HOUR: u16 = crate::misc::int_conv::u8u16(SECONDS_PER_MINUTE) * 60;
pub(crate) const SECONDS_PER_MINUTE: u8 = 60;
pub(crate) const YEARS_PER_QUADCENTURY: u16 = 400;

pub(crate) static DAYS_OF_MONTHS: [[u16; 12]; 2] = [
  [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
  [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
];
#[cfg(feature = "epoch-sync")]
pub(crate) static EPOCH_OFFSET: epoch_offset::EpochOffset = epoch_offset::EpochOffset::new();

/// Used by embedded devices in `no_std` scenarios where the timer only provides the elapsed
/// time since boot.
///
/// It is still necessary to figure out the IP address of a NTP pool before entering in this
/// function.
///
/// Returns `false` if the server didn't respond within 5 seconds.
#[cfg(feature = "epoch-sync")]
#[inline]
pub async fn fetch_and_set_epoch_offset<S>(
  addr: core::net::SocketAddr,
  stream: &mut S,
) -> crate::Result<bool>
where
  S: crate::net::UdpStream,
{
  use crate::futures::Timeout;
  use core::time::Duration;

  let mut buffer = [0u8; 48];
  buffer[0] = 0b0010_0011;
  drop(Timeout::new(stream.send_to(&mut buffer, addr), Duration::from_secs(5))?.await?);
  drop(Timeout::new(stream.recv_from(&mut buffer), Duration::from_secs(5))?.await?);
  let seconds_bytes: [u8; 4] = buffer[40..44].try_into().unwrap_or_default();
  let ntp_seconds = u32::from_be_bytes(seconds_bytes);
  EPOCH_OFFSET.set(ntp_seconds.into())?;
  let mut orig_buffer = [0u8; 48];
  orig_buffer[0] = 0b0010_0011;
  Ok(buffer != orig_buffer)
}

/// The current time in according to `cb` as a string.
#[inline]
pub fn timestamp_str(
  cb: impl FnOnce(core::time::Duration) -> u128,
) -> crate::Result<(u64, U64String)> {
  let number = Instant::now_timestamp().map(cb)?.try_into()?;
  Ok((number, u64_string(number)))
}

#[cfg(test)]
mod tests {
  #[cfg(all(feature = "embassy-time", feature = "_hack", feature = "_integration-tests"))]
  #[wtx::test]
  async fn fetch_and_set_epoch_offset_works_as_expected() {
    use crate::calendar::{EPOCH_OFFSET, Instant, fetch_and_set_epoch_offset};
    use std::net::{ToSocketAddrs, UdpSocket};

    assert_eq!(EPOCH_OFFSET.get(), 0);
    let mut udp_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let addr = "pool.ntp.org:123".to_socket_addrs().unwrap().into_iter().next().unwrap();
    let _ = fetch_and_set_epoch_offset(addr, &mut udp_socket).await.unwrap();
    assert!(EPOCH_OFFSET.get() > 1000000);
    assert!(Instant::now_date_time().unwrap().timestamp_secs_and_ns().0 > 1784244618);
  }
}
