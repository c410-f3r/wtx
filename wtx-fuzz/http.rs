//! HTTP

#![no_main]

use core::str;
use wtx::{
  collections::VectorUsize,
  http::{HeaderName, KnownHeaderName, Method, Protocol, Version, cookie::CookieStr},
};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  // Generic
  {
    let _rslt0 = Method::from_canonical_bytes(data);
    let _rslt1 = Protocol::from_canonical_bytes(data);
    let _rslt2 = Version::try_from(data);
    let _rslt3 = KnownHeaderName::try_from(data);
    let _rslt4 = HeaderName::from_checked_bytes(data);
  }

  // Cookie
  {
    if let Ok(input) = str::from_utf8(data) {
      let mut buffer = VectorUsize::new();
      let _rslt = CookieStr::parse(&mut buffer, input);
    }
  }
});
