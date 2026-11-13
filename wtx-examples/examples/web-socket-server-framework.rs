//! High-level WebSocket server that supports the routing of paths.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  collections::VectorUsize,
  http::WebSocketServerFramework,
  tls::{SkCtx, TlsConfig},
  web_socket::WebSocket,
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

type LocalWebSocket = WebSocket<(), TcpStream, SkCtx, false>;

fn main() -> wtx::Result<()> {
  let tls_config = TlsConfig::from_keys_pem(PUBLIC_KEY, SECRET_KEY)?;
  let router = (("/echo", echo),);
  WebSocketServerFramework::tokio(tls_config)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
}

async fn echo(mut buffer: VectorUsize<u8>, mut ws: LocalWebSocket) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  loop {
    let Ok(frames) = reader.read_frames(&mut buffer, &mut common).await else {
      return Ok(());
    };
    writer.write_frames(&mut common, frames.filter(|el| el.op_code().is_data())).await?;
  }
}
