//! Basic time operation.

extern crate wtx;

use wtx::calendar::{Duration, Instant};

fn main() -> wtx::Result<()> {
  println!(
    "ISO 8601 representation of the next 2 minutes in UTC: {}",
    Instant::now_date_time(0)?.add(Duration::from_minutes(2)?)?
  );
  Ok(())
}
