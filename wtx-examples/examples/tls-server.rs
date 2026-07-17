//! TLS server that performs handshakes and replies application data.
//!
//! There is no HTTP, WebSocket or any other protocol, only TLS!

extern crate tokio;
extern crate wtx;

use tokio::net::{TcpListener, TcpStream};
use wtx::{
  misc::SecretContext,
  rng::{ChaCha20, CryptoSeedableRng as _},
  stream::{StreamReader, StreamWriter},
  tls::{TlsAcceptor, TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(&host_from_args()).await?;
  loop {
    let (stream, _) = listener.accept().await?;
    tokio::spawn(async move {
      if let Err(err) = connection(stream).await {
        eprintln!("Error: {err:?}");
      }
    });
  }
}

async fn connection(stream: TcpStream) -> wtx::Result<()> {
  let mut rng = ChaCha20::from_std_random()?;
  let secret_context = SecretContext::new(&mut rng)?;
  let tls_config = TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    PUBLIC_KEY.try_into()?,
    &mut rng,
    (secret_context, &mut SECRET_KEY.clone()),
  )?;
  let tls_connector = TlsAcceptor::new(tls_config, rng, stream);
  let mut tls_stream = tls_connector.accept().await?.tls_stream;
  loop {
    let mut bytes = [0u8; 1024];
    let Some(read) = tls_stream.read(bytes.as_mut().into()).await? else {
      break;
    };
    let slice = bytes.get(..read.get()).unwrap_or_default();
    tls_stream.write_all(slice).await?;
  }
  Ok(())
}
