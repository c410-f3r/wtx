//! Basic time operations.

extern crate wtx;

use core::time::Duration;
use wtx::time::{DateTime, Instant};

fn main() -> wtx::Result<()> {
  let now = Instant::now();
  let now_plus_2_minutes = now.checked_add(Duration::from_secs(60 * 2))?;
  let timestamp = now_plus_2_minutes.timestamp()?.as_secs().cast_signed();
  let date_time = DateTime::from_timestamp_secs(timestamp)?;
  println!("ISO 8601 representation of the next 2 minutes in UTC: {}", date_time);
  Ok(())
}
