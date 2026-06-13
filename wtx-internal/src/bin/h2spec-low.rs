//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use core::mem;
use std::net::TcpListener;
use wtx::{
  collection::Vector,
  executor::StdRuntime,
  http::{HttpRecvParams, StatusCode},
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
            let mut enc_buffer = Vector::new();
            let mut common = http2_stream.common();
            let fun = async {
              loop {
                let hrs = common.recv_data(|_| Ok(())).await?;
                match hrs {
                  Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) => {
                    return Ok(());
                  }
                  Http2RecvStatus::Eos(_) => break,
                  Http2RecvStatus::Ongoing(_) => continue,
                }
              }
              let _ = common.recv_trailers().await?;
              let _ = common.send_headers(&mut enc_buffer, &headers, false, StatusCode::Ok).await?;
              let _ = common.send_data(b"Hello", true).await?;
              common.clear().await?;
              wtx::Result::Ok(())
            };
            if let Err(err) = fun.await {
              http2_stream.common().send_go_away(Http2ErrorCode::InternalError).await;
              eprint!("{err}");
            }
          });
        }
      };
      if let Err(err) = fun.await {
        eprint!("{err}");
      }
    });
  }
}
