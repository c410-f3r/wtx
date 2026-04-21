//! WebSocket autobahn client.

use wtx::{
  collection::Vector,
  web_socket::{Frame, OpCode, WebSocketPayloadOrigin},
};
use wtx_internal::{autobahn_case_conn, autobahn_close, autobahn_get_case_count};

#[wtx::main]
async fn main() -> wtx::Result<()> {
  let host = "127.0.0.1:9080";
  let mut buffer = Vector::new();
  for case in 1..=autobahn_get_case_count(&mut buffer, host).await.unwrap() {
    let mut ws = autobahn_case_conn(case, host).await.unwrap();
    let (mut common, mut reader, mut writer) = ws.split_mut();
    loop {
      let mut frame =
        match reader.read_frame(&mut buffer, &mut common, WebSocketPayloadOrigin::Adaptive).await {
          Err(_err) => {
            ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await.unwrap();
            break;
          }
          Ok(elem) => elem,
        };
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await.unwrap(),
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  autobahn_close(host).await
}
