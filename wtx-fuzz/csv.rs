//! CSV

#![no_main]

use std::io::Cursor;
use wtx::{codec::Csv, collections::VectorUsize};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  let mut csv = Csv::from_buf_read(Cursor::new(data));
  let mut buffer = VectorUsize::new();
  loop {
    match csv.next_elements(&mut buffer) {
      Ok(Some(elements)) => for _element in elements {},
      _ => break,
    }
  }
});
