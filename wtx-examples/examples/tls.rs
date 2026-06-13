//! Behold the thing that most servers try to minimize: A TLS handshake 👻
//!
//! Ever heard about 0-RTT, PSK or signatureless certificates? If not, then you probably are a
//! happy and healthy individual.

extern crate wtx;

use std::net::TcpStream;
use wtx::{
  collection::Vector,
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsConnector},
};
use wtx_examples::LocalTlsMode;

fn main() -> wtx::Result<()> {
  let stream = TcpStream::connect("https://github.com/c410-f3r/wtx")?;
  let tls_config = TlsConfig::from_ccadb();
  let mut rng = ChaCha20::from_getrandom()?;
  let mut tls_connector = TlsConnector::from_stream(stream).tls_mode(LocalTlsMode::default());
  tls_connector.write_client_hello(None, &mut rng, &tls_config, &mut Vector::new())?;
  Ok(())
}
