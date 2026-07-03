//! TLS client that reads and writes records.

extern crate tokio;
extern crate wtx;

use tokio::net::TcpStream;
use wtx::{
  rng::{ChaCha20, CryptoSeedableRng as _},
  stream::{StreamReader, StreamWriter},
  tls::{TlsConfig, TlsConnector, TlsModeVerified},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let stream = TcpStream::connect("github.com:443").await?;
  let tls_config = TlsConfig::from_ccadb(TlsModeVerified::default())?;
  let tls_connector = TlsConnector::new(tls_config, ChaCha20::from_getrandom()?, stream);
  let mut tls_stream = tls_connector.connect().await?.rslt()?.tls_stream;
  let request = b"GET /c410-f3r/wtx HTTP/1.1\r\nHost: github.com\r\nConnection: close\r\n\r\n";
  tls_stream.write_all(request).await?;
  loop {
    let mut buffer = [0; 128];
    let Some(read) = tls_stream.read(buffer.as_mut_slice().into()).await?.opt() else {
      return Ok(());
    };
    println!("Received data: {:?}", buffer.get(..read.get()).unwrap_or_default());
  }
}
