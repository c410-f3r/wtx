//! Calendar

#![no_main]

use wtx::calendar::{Date, Datetime, Time, Utc};

libfuzzer_sys::fuzz_target!(|data: (&[u8], &[u8])| {
  let (value, fmt) = data;

  let _rslt0 = Date::from_iso8601(value);
  let _rslt1 = Time::from_iso8601(value);
  let _rslt2 = Datetime::<Utc>::from_iso8601(value);

  if let Ok(bytes) = <[u8; 8]>::try_from(value.get(..8).unwrap_or_default()) {
    let _rslt3 = Datetime::<Utc>::from_timestamp_secs(i64::from_ne_bytes(bytes));
  }

  for chunk in fmt.as_chunks::<2>().0.into_iter().copied() {
    let Ok(token) = chunk.try_into() else {
      continue;
    };
    let _rslt4 = Datetime::<Utc>::parse(&value, [token]);
  }
});
