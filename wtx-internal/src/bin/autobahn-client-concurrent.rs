//! WebSocket autobahn client.

use core::pin::pin;
use wtx::{
  collections::Vector,
  futures::PollOnce,
  web_socket::{Frame, OpCode, WebSocketPayloadOrigin},
};
use wtx_internal::{autobahn_case_conn, autobahn_close, autobahn_get_case_count};

#[tokio::main]
async fn main() {
  let host = "127.0.0.1:9080";
  let mut buffer = Vector::new();
  for case in 1..=autobahn_get_case_count(&mut buffer, host).await.unwrap() {
    let ws = autobahn_case_conn(case, host).await.unwrap();
    let (stream_bridge, mut stream_reader, mut stream_writer) = ws.into_split().unwrap();
    let mut bridge_frame = pin!(stream_bridge.listen());
    loop {
      let mut frame =
        match stream_reader.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await {
          Err(_err) => {
            stream_writer
              .write_frame(&mut Frame::new_fin(OpCode::Close, &mut []).unwrap())
              .await
              .unwrap();
            break;
          }
          Ok(elem) => elem,
        };
      if let Some(el) = PollOnce::new(&mut bridge_frame).await {
        if stream_writer.manage_bridge_data(el).await.unwrap() {
          break;
        }
        bridge_frame.set(stream_bridge.listen());
      }
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => stream_writer.write_frame(&mut frame).await.unwrap(),
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  autobahn_close(host).await.unwrap()
}
