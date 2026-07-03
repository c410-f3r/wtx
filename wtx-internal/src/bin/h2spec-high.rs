//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use core::mem;
use tokio::net::TcpListener;
use wtx::{
  collections::Vector,
  http::{HttpRecvParams, Response, StatusCode},
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2RecvStatus},
  rng::{ChaCha20, CryptoSeedableRng, Xorshift64},
  stream::Stream,
  tls::{TlsAcceptor, TlsConfig},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:9000").await?;
  loop {
    let (stream, _) = listener.accept().await?;
    let _conn_jh = tokio::spawn(async move {
      let fun = async {
        let rng = &mut ChaCha20::from_std_random().unwrap();
        let http2_params = HttpRecvParams::with_default_params();
        let http2_buffer = Http2Buffer::new(&mut Xorshift64::from_std_random()?);
        let tls_stream = TlsAcceptor::new(&TlsConfig::empty(), rng, stream)
          .accept()
          .await
          .unwrap()
          .rslt()
          .unwrap()
          .tls_stream;
        let http2 = Http2::accept(http2_buffer, http2_params, tls_stream.into_split()?).await?;
        let (frame_reader, http2) = http2;
        let _frame_reader_jh = tokio::spawn(frame_reader);
        loop {
          let stream = match http2.stream(|req, _| mem::take(&mut req.msg_data.headers)).await? {
            None => return wtx::Result::Ok(()),
            Some(elem) => elem,
          };
          let (mut http2_stream, headers) = stream;
          let _stream_jh = tokio::spawn(async move {
            let fun = async {
              let (hrs, _) = http2_stream.recv_req().await?;
              if let Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) = hrs {
                return Ok(());
              }
              let res = Response::new(("Hello", &headers), StatusCode::Ok);
              let _ = http2_stream.send_res(&mut Vector::new(), res).await?;
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
