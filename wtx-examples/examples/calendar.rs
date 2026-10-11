//! Basic time operation.

extern crate wtx;

use wtx::calendar::{IDuration, Instant};

fn main() -> wtx::Result<()> {
  println!(
    "ISO 8601 representation of the next 2 minutes in UTC: {}",
    Instant::now_datetime()?.add(IDuration::from_minutes(2)?)?
  );
  Ok(())
}
