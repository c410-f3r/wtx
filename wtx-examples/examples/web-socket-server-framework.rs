//! High-level WebSocket server that supports the routing of paths.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  executor::TokioExecutor,
  http::WebSocketServerFramework,
  tls::TlsConfig,
  web_socket::{OpCode, WebSocket, WebSocketBuffer, WebSocketPayloadOrigin},
};
use wtx_examples::{LocalTlsMode, PUBLIC_KEY, SECRET_KEY};

type LocalWebSocket = WebSocket<(), TcpStream, LocalTlsMode, WebSocketBuffer, false>;

fn main() -> wtx::Result<()> {
  WebSocketServerFramework::new(TokioExecutor)?
    .set_tls_config(TlsConfig::from_keys(PUBLIC_KEY, SECRET_KEY))
    .set_tls_mode(LocalTlsMode::default())
    .run_in_threads(&wtx_examples::host_from_args(), (("/echo", echo),))
}

async fn echo(mut buffer: Vector<u8>, mut ws: LocalWebSocket) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  loop {
    let origin = WebSocketPayloadOrigin::Adaptive;
    let mut frame = reader.read_frame(&mut buffer, &mut common, origin).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await?,
      OpCode::Close => return Ok(()),
      _ => {}
    }
  }
}
