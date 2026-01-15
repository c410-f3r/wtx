//! Pure TLS client intended for testing purposes.

use std::net::TcpStream;

use wtx::{
  misc::Uri,
  rng::{ChaCha20, SeedableRng},
  tls::{TlsConfig, TlsConnector},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("localhost:9000");
  let mut rng = ChaCha20::from_getrandom()?;
  let _tls_stream = TlsConnector::default()
    .connect(&mut rng, TcpStream::connect(uri.hostname_with_implied_port())?, &TlsConfig::default())
    .await?;
  Ok(())
}
