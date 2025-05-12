//! DateTime

#![no_main]

use wtx::time::DateTime;

libfuzzer_sys::fuzz_target!(|(value, fmt): (Vec<u8>, Vec<u8>)| {
  let _rslt = DateTime::parse(&value, &value);
});
