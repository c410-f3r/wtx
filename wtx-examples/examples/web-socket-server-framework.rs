use tokio::net::TcpStream;
use wtx::{
  collections::VectorUsize,
  http::WebSocketServerFramework,
  net::TcpParams,
  tls::{PlaintextCtx, TlsConfig},
  web_socket::{WebSocket, WebSocketPayloadOrigin},
};

type LocalWebSocket = WebSocket<(), TcpStream, PlaintextCtx, false>;

fn main() -> wtx::Result<()> {
  WebSocketServerFramework::tokio(TlsConfig::plaintext())?
    .set_tcp_params(TcpParams::default())
    .run_in_threads("0.0.0.0:8080", (("/ws", ws),))
}

async fn ws(mut buffer: VectorUsize<u8>, mut ws: LocalWebSocket) -> wtx::Result<()> {
  let (mut common, mut reader, mut writer) = ws.split_mut();
  loop {
    let origin = WebSocketPayloadOrigin::Adaptive;
    let Ok(frames) = reader.read_frames(&mut buffer, &mut common, origin).await else {
      return Ok(());
    };
    writer.write_frames(&mut common, frames).await?;
  }
}
