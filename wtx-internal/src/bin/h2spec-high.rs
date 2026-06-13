//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use core::mem;
use std::net::TcpListener;
use wtx::{
  executor::StdRuntime,
  http::{HttpRecvParams, Response, StatusCode},
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2RecvStatus},
  rng::{CryptoSeedableRng, Xorshift64},
  sync::Arc,
};

#[wtx::main]
async fn main(runtime: Arc<StdRuntime>) -> wtx::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:9000")?;
  loop {
    let (tcp_stream, _) = listener.accept()?;
    let conn_runtime = runtime.clone();
    let _conn_jh = runtime.spawn_threaded(async move {
      let fun = async {
        let http2_params = HttpRecvParams::with_default_params();
        let http2_buffer = Http2Buffer::new(&mut Xorshift64::from_std_random()?);
        let parts = (tcp_stream.try_clone().unwrap(), tcp_stream);
        let http2 = Http2::accept(http2_buffer, http2_params, parts).await?;
        let (frame_reader, http2) = http2;
        let _frame_reader_jh = conn_runtime.spawn_threaded(frame_reader)?;
        loop {
          let stream = match http2.stream(|req, _| mem::take(&mut req.msg_data.headers)).await? {
            None => return wtx::Result::Ok(()),
            Some(elem) => elem,
          };
          let (mut http2_stream, headers) = stream;
          let _stream_jh = conn_runtime.spawn_threaded(async move {
            let fun = async {
              let (hrs, _) = http2_stream.recv_req().await?;
              if let Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) = hrs {
                return Ok(());
              }
              let _ =
                http2_stream.send_res(Response::new(("Hello", &headers), StatusCode::Ok)).await?;
              wtx::Result::Ok(())
            };
            if let Err(err) = fun.await {
              http2_stream.common().send_go_away(Http2ErrorCode::InternalError).await;
              eprint!("{err}");
            }
          })?;
        }
      };
      if let Err(err) = fun.await {
        eprint!("{err}");
      }
    });
  }
}
