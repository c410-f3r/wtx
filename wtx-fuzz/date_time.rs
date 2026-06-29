//! DateTime

#![no_main]

use wtx::calendar::{DateTime, Utc};

libfuzzer_sys::fuzz_target!(|data: (Vec<u8>, Vec<u8>)| {
  let (value, fmt) = data;
  for chunk in fmt.as_chunks::<2>().0.into_iter().copied() {
    let Ok(token) = chunk.try_into() else {
      continue;
    };
    let _rslt = DateTime::<Utc>::parse(&value, [token]);
  }
});
