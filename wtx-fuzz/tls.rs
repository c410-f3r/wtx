//! TLS

#![no_main]

use wtx::{
  executor::StdRuntime,
  net::{BytesStream, StreamWriter as _, UriRef},
  rng::ChaCha20,
  tls::{TlsConfig, TlsConnector},
};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  let runtime = StdRuntime::new();
  let mut stream = BytesStream::default();
  let _rslt0 = runtime.block_on(stream.write_all(data));
  let connector = TlsConnector::new(
    TlsConfig::unverified(),
    ChaCha20::from_key([0; 32]),
    stream,
    UriRef::new("https://example.com"),
  );
  let _rslt1 = runtime.block_on(connector.connect());
});
