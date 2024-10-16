//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use std::mem;

use tokio::net::TcpListener;
use wtx::{
  http::ReqResBuffer,
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2RecvStatus, Http2Tokio},
  misc::{simple_seed, Either, Vector, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:9000").await?;
  loop {
    let (tcp_stream, _) = listener.accept().await?;
    let _conn_jh = tokio::spawn(async move {
      let fun = async {
        let http2_params = Http2Params::default();
        let http2_buffer = Http2Buffer::new(Xorshift64::from(simple_seed()));
        let tuple =
          Http2Tokio::accept(http2_buffer, (), http2_params, tcp_stream.into_split()).await?;
        let (frame_reader, mut http2) = tuple;
        let _jh = tokio::spawn(frame_reader);
        loop {
          let rslt = http2.stream(ReqResBuffer::default(), |headers| mem::take(headers)).await;
          let (mut http2_stream, mut headers) = match rslt? {
            Either::Left(_) => return wtx::Result::Ok(()),
            Either::Right(elem) => elem,
          };
          let _stream_jh = tokio::spawn(async move {
            let fun = async {
              let mut body = Vector::new();
              loop {
                let (hrs, local_body) = http2_stream.fetch_data(body).await?;
                body = local_body;
                match hrs {
                  Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream => {
                    return Ok(())
                  }
                  Http2RecvStatus::Eos => break,
                  Http2RecvStatus::Ok(_) => continue,
                }
              }
              loop {
                let (hrs, local_headers) = http2_stream.fetch_trailers(headers).await?;
                headers = local_headers;
                match hrs {
                  Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream => {
                    return Ok(())
                  }
                  Http2RecvStatus::Eos => break,
                  Http2RecvStatus::Ok(_) => continue,
                }
              }
              let _ = http2_stream.send_data(b"Hello", true).await?;
              wtx::Result::Ok(())
            };
            if let Err(err) = fun.await {
              http2_stream.send_go_away(Http2ErrorCode::InternalError).await;
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
