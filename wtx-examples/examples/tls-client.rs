//! TLS client that reads and writes records.

extern crate tokio;
extern crate wtx;

use wtx::{
  misc::{Uri, process_utf8_stream},
  rng::{ChaCha20, CryptoSeedableRng as _},
  stream::{StreamReader, StreamWriter},
  tls::{TlsConfig, TlsConnectorBuilder, TlsModeVerified},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("github.com:443");
  let mut tls_stream = TlsConnectorBuilder::tokio(uri)
    .build(TlsConfig::from_ccadb(TlsModeVerified::default())?, ChaCha20::from_getrandom()?)
    .await?
    .connect()
    .await?
    .tls_stream;
  let request = b"GET /c410-f3r/wtx HTTP/1.1\r\nHost: github.com\r\nConnection: close\r\n\r\n";
  tls_stream.write_all(request).await?;
  let mut partial_char = None;
  loop {
    let mut buffer = [0; 128];
    let Some(read) = tls_stream.read(buffer.as_mut_slice().into()).await? else {
      return Ok(());
    };
    let slice = buffer.get(..read.get()).unwrap_or_default();
    let (lhs, rhs) = process_utf8_stream(&mut partial_char, slice)?;
    println!("{lhs}{rhs}");
  }
}
