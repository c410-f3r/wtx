//! WebSocket autobahn client.

use core::pin::pin;
use wtx::{
  collection::Vector,
  misc::PollOnce,
  web_socket::{Frame, OpCode, WebSocketPartsOwned, WebSocketPayloadOrigin},
};
use wtx_instances::{autobahn_case_conn, autobahn_close, autobahn_get_case_count};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let host = "127.0.0.1:9080";
  let mut buffer = Vector::new();
  for case in 1..=autobahn_get_case_count(&mut buffer, host).await? {
    let ws = autobahn_case_conn(case, host).await?;
    let WebSocketPartsOwned { mut reader, replier, mut writer } =
      ws.into_split(tokio::io::split)?;
    let mut reply_frame = pin!(replier.reply_frame());
    loop {
      let mut frame = match reader.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await {
        Err(_err) => {
          writer.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await?;
          break;
        }
        Ok(elem) => elem,
      };
      if let Some(mut elem) = PollOnce::new(&mut reply_frame).await {
        if writer.write_reply_frame(&mut elem).await? {
          break;
        }
        reply_frame.set(replier.reply_frame());
      }
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => writer.write_frame(&mut frame).await?,
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  autobahn_close(host).await
}
