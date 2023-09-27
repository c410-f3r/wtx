//! Profiling

use std::hint::black_box;
use wtx::{
  rng::StaticRng,
  web_socket::{FrameBufferVec, FrameMutVec, OpCode, WebSocket, WebSocketClient, WebSocketServer},
  BytesStream, PartitionedBuffer,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> wtx::Result<()> {
  let data = vec![52; 16 * 1024 * 1024];
  let mut fb = FrameBufferVec::default();
  let mut stream = BytesStream::default();
  black_box(from_client_to_server(&data, &mut fb, &mut stream).await?);
  stream.clear();
  black_box(from_server_to_client(&data, &mut fb, &mut stream).await?);
  Ok(())
}

async fn from_client_to_server(
  data: &[u8],
  fb: &mut FrameBufferVec,
  stream: &mut BytesStream,
) -> wtx::Result<()> {
  write(data, fb, WebSocketClient::new((), <_>::default(), StaticRng::default(), stream)).await?;
  read(fb, WebSocketServer::new((), <_>::default(), StaticRng::default(), stream)).await?;
  Ok(())
}

async fn from_server_to_client(
  data: &[u8],
  fb: &mut FrameBufferVec,
  stream: &mut BytesStream,
) -> wtx::Result<()> {
  write(data, fb, WebSocketServer::new((), <_>::default(), StaticRng::default(), stream)).await?;
  read(fb, WebSocketClient::new((), <_>::default(), StaticRng::default(), stream)).await?;
  Ok(())
}

#[allow(
  // False positive
  clippy::needless_pass_by_ref_mut
)]
async fn read<const IS_CLIENT: bool>(
  fb: &mut FrameBufferVec,
  mut ws: WebSocket<(), PartitionedBuffer, StaticRng, &mut BytesStream, IS_CLIENT>,
) -> wtx::Result<()> {
  let _frame = ws.read_frame(fb).await?;
  Ok(())
}

async fn write<const IS_CLIENT: bool>(
  data: &[u8],
  fb: &mut FrameBufferVec,
  mut ws: WebSocket<(), PartitionedBuffer, StaticRng, &mut BytesStream, IS_CLIENT>,
) -> wtx::Result<()> {
  ws.write_frame(&mut FrameMutVec::new_unfin(fb, OpCode::Text, data)?).await?;
  ws.write_frame(&mut FrameMutVec::new_unfin(fb, OpCode::Continuation, data)?).await?;
  ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Continuation, data)?).await?;
  Ok(())
}
