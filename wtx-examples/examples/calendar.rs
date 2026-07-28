//! Basic time operation.

extern crate wtx;

use wtx::calendar::{Instant, SigDuration};

fn main() -> wtx::Result<()> {
  println!(
    "ISO 8601 representation of the next 2 minutes in UTC: {}",
    Instant::now_date_time()?.add(SigDuration::from_minutes(2)?)?
  );
  Ok(())
}
