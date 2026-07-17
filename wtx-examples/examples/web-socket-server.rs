//! Serves requests using low-level WebSockets resources alongside self-made certificates.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpListener;
use wtx::{
  collections::Vector,
  misc::SecretContext,
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsAcceptor, TlsConfig, TlsModeVerified},
  web_socket::{OpCode, WebSocketAcceptor, WebSocketPayloadOrigin},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(&host_from_args()).await?;
  let mut rng = ChaCha20::from_getrandom()?;
  let secret_context = SecretContext::new(&mut rng)?;
  loop {
    let mut conn_rng = ChaCha20::from_crypto_rng(&mut rng)?;
    let conn_secret_context = secret_context.clone();
    let (stream, _) = listener.accept().await?;
    let _jh = tokio::spawn(async move {
      let fut = async {
        let tls_config = TlsConfig::from_keys_pem(
          TlsModeVerified::default(),
          PUBLIC_KEY.try_into()?,
          &mut conn_rng,
          (conn_secret_context, &mut SECRET_KEY.clone()),
        )?;
        let mut buffer = Vector::new();
        let mut ws = WebSocketAcceptor::default()
          .accept(TlsAcceptor::new(tls_config, conn_rng, stream))
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
