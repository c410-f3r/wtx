//! Basic time operation.

extern crate wtx;

use wtx::calendar::{Datetime, IDuration};

fn main() -> wtx::Result<()> {
  println!(
    "ISO 8601 representation of the next 2 minutes in UTC: {}",
    Datetime::now()?.add(IDuration::from_minutes(2)?)?
  );
  Ok(())
}
