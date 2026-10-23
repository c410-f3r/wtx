//! Base64

#![no_main]

use wtx::codec::{
  Base64Alphabet, base64_decode, base64_decoded_len_ub, base64_encode, base64_encoded_len,
};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  for elem in [
    Base64Alphabet::Standard,
    Base64Alphabet::StandardNoPad,
    Base64Alphabet::Url,
    Base64Alphabet::UrlNoPad,
  ] {
    let mut decoded = vec![0; base64_decoded_len_ub(data.len())];
    let Ok(decoded) = base64_decode(elem, data, &mut decoded) else {
      continue;
    };
    let Some(encoded_len) = base64_encoded_len(decoded.len(), elem.is_padded()) else {
      continue;
    };
    let mut encoded = vec![0; encoded_len];
    let _rslt = base64_encode(elem, decoded, &mut encoded);
  }
});
