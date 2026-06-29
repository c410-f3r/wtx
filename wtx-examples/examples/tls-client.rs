//! Behold the thing that most servers try to minimize: A TLS handshake 👻
//!
//! Ever heard about 0-RTT, PSK or signatureless certificates? If not, then you probably are a
//! happy and healthy individual.

extern crate wtx;

use std::net::TcpStream;
use wtx::{
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsConnector, TlsModeVerified},
};

fn main() -> wtx::Result<()> {
  let stream = TcpStream::connect("github.com:443")?;
  let tls_config = TlsConfig::from_ccadb(TlsModeVerified::default())?;
  let mut tls_connector = TlsConnector::new(tls_config, ChaCha20::from_getrandom()?, stream);
  tls_connector.write_client_hello()?;
  Ok(())
}
