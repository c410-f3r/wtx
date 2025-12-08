//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use core::mem;
use tokio::net::TcpListener;
use wtx::{
  collection::Vector,
  http::StatusCode,
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2Params, Http2RecvStatus},
  rng::{Xorshift64, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:9000").await?;
  loop {
    let (tcp_stream, _) = listener.accept().await?;
    let _conn_jh = tokio::spawn(async move {
      let fun = async {
        let http2_params = Http2Params::default();
        let http2_buffer = Http2Buffer::new(&mut Xorshift64::from(simple_seed()));
        let tuple = Http2::accept(http2_buffer, http2_params, tcp_stream.into_split()).await?;
        let (frame_reader, http2) = tuple;
        let _jh = tokio::spawn(frame_reader);
        loop {
          let (mut http2_stream, headers) =
            match http2.stream(|req, _| mem::take(&mut req.rrd.headers)).await? {
              None => return wtx::Result::Ok(()),
              Some(elem) => elem,
            };
          let _stream_jh = tokio::spawn(async move {
            let mut enc_buffer = Vector::new();
            let mut common = http2_stream.common();
            let fun = async {
              loop {
                let hrs = common.recv_data().await?;
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
              common.clear(true).await?;
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
