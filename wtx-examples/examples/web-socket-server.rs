//! Serves requests using low-level WebSockets resources alongside self-made certificates.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpListener;
use wtx::{
  collections::Vector,
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsAcceptor, TlsConfig, TlsModeVerified},
  web_socket::{OpCode, WebSocketAcceptor, WebSocketPayloadOrigin},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(&host_from_args()).await?;
  let mut rng = ChaCha20::from_getrandom()?;
  loop {
    let conn_rng = ChaCha20::from_crypto_rng(&mut rng)?;
    let (stream, _) = listener.accept().await?;
    let _jh = tokio::spawn(async move {
      let fut = async {
        let mut buffer = Vector::new();
        let mut ws = WebSocketAcceptor::default()
          .accept(TlsAcceptor::new(
            TlsConfig::from_keys_pem(TlsModeVerified::default(), PUBLIC_KEY, SECRET_KEY)?,
            conn_rng,
            stream,
          ))
          .await?;
        let (mut common, mut reader, mut writer) = ws.split_mut();
        loop {
          let origin = WebSocketPayloadOrigin::Adaptive;
          let mut frame = reader.read_frame(&mut buffer, &mut common, origin).await?;
          match frame.op_code() {
            OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await?,
            OpCode::Close => break,
            _ => {}
          }
        }
        wtx::Result::Ok(())
      };
      if let Err(err) = fut.await {
        eprintln!("Error: {err}");
      }
    });
  }
}
