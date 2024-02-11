//! WebSocket benchmark

use crate::misc::Agent;
use std::time::Instant;
use tokio::{net::TcpStream, task::JoinSet};
use wtx::{
  misc::UriRef,
  rng::StaticRng,
  web_socket::{
    handshake::{WebSocketConnect, WebSocketConnectRaw},
    FrameBufferVec, FrameMutVec, OpCode, WebSocketBuffer,
  },
};

// Verifies the handling of concurrent calls.
const CONNECTIONS: usize = 2048;
// Bytes to receive and send
const FRAME_DATA: &[u8; FRAME_LEN] = &[53; FRAME_LEN];
// Some applications use WebSocket to perform streaming so the length of a frame can be quite large
// but statistically it is generally low.
const FRAME_LEN: usize = 64 * 1024;
// For each message, the client always verifies the content sent back from a server and this
// leads to a sequential-like behavior.
//
// If this is the only high metric, all different servers end-up performing similarly effectively
// making this criteria an "augmenting factor" when combined with other parameters.
const NUM_MESSAGES: usize = 16;
/// A set of frames composes a message.
const NUM_FRAMES: usize = {
  let n = NUM_MESSAGES / 4;
  if n == 0 {
    1
  } else {
    n
  }
};

pub(crate) async fn bench(agent: &mut Agent, uri: &UriRef<'_>) {
  let instant = Instant::now();
  let mut set = JoinSet::new();
  for _ in 0..CONNECTIONS {
    let _handle = set.spawn({
      let local_uri = uri.to_string();
      async move {
        let fb = &mut FrameBufferVec::default();
        let (_, mut ws) = WebSocketConnectRaw {
          compression: (),
          fb,
          headers_buffer: &mut <_>::default(),
          rng: StaticRng::default(),
          stream: TcpStream::connect(local_uri.authority()).await.unwrap(),
          uri: &local_uri.to_ref(),
          wsb: WebSocketBuffer::default(),
        }
        .connect([])
        .await
        .unwrap();
        for _ in 0..NUM_MESSAGES {
          match NUM_FRAMES {
            0 => break,
            1 => {
              ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, FRAME_DATA).unwrap())
                .await
                .unwrap();
            }
            _ => {
              ws.write_frame(&mut FrameMutVec::new_unfin(fb, OpCode::Text, FRAME_DATA).unwrap())
                .await
                .unwrap();
              for _ in (0..NUM_FRAMES).skip(2) {
                ws.write_frame(
                  &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, FRAME_DATA).unwrap(),
                )
                .await
                .unwrap();
              }
              ws.write_frame(
                &mut FrameMutVec::new_fin(fb, OpCode::Continuation, FRAME_DATA).unwrap(),
              )
              .await
              .unwrap();
            }
          }
          assert_eq!(ws.read_frame(fb).await.unwrap().fb().payload().len(), FRAME_LEN * NUM_FRAMES);
        }
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[]).unwrap()).await.unwrap();
      }
    });
  }
  while let Some(rslt) = set.join_next().await {
    rslt.unwrap();
  }
  agent.result = instant.elapsed().as_millis();
}

pub(crate) fn caption() -> String {
  format!("{CONNECTIONS} connection(s) sending {NUM_MESSAGES} message(s) composed by {NUM_FRAMES} frame(s) of {FRAME_LEN} byte(s)")
}
