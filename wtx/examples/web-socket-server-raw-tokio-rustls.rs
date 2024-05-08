//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use tokio::net::TcpListener;
use wtx::{
  misc::TokioRustlsAcceptor,
  rng::StdRng,
  web_socket::{
    handshake::{WebSocketAccept, WebSocketAcceptRaw},
    FrameBufferVec, OpCode, WebSocketBuffer,
  },
};

static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
static KEY: &[u8] = include_bytes!("../../.certs/key.pem");

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  let acceptor = TokioRustlsAcceptor::default().with_cert_chain_and_priv_key(CERT, KEY).unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let local_acceptor = acceptor.clone();
    let _jh = tokio::spawn(async move {
      let tls_stream = local_acceptor.accept(stream).await.unwrap();
      let fun = || async move {
        let mut ws = WebSocketAcceptRaw {
          compression: (),
          rng: StdRng::default(),
          stream: tls_stream,
          wsb: WebSocketBuffer::default(),
        }
        .accept(|_| true)
        .await?;
        let mut fb = FrameBufferVec::default();
        loop {
          let mut frame = ws.read_frame(&mut fb).await?;
          match frame.op_code() {
            OpCode::Binary | OpCode::Text => {
              ws.write_frame(&mut frame).await?;
            }
            OpCode::Close => break,
            _ => {}
          }
        }
        wtx::Result::Ok(())
      };
      if let Err(err) = fun().await {
        eprintln!("{err:?}");
      }
    });
  }
}
