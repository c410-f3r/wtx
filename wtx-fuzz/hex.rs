//! Hex

#![no_main]

use wtx::codec::{HexEncMode, hex_decode, hex_encode};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  let mut decoded = vec![0; data.len() / 2];
  let Ok(decoded) = hex_decode(data, &mut decoded) else {
    return;
  };
  let mut encoded = vec![0; decoded.len() * 2 + 2];
  for mode in [
    HexEncMode::WithPrefixLower,
    HexEncMode::WithPrefixUpper,
    HexEncMode::WithoutPrefixLower,
    HexEncMode::WithoutPrefixUpper,
  ] {
    let _rslt = hex_encode(decoded, Some(mode), &mut encoded);
  }
});
