//! h2spec

#![expect(clippy::print_stderr, reason = "internal")]

use tokio::net::TcpListener;
use wtx::{
  http::{ReqResBuffer, StatusCode},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{simple_seed, Either, Xorshift64},
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
        let tuple = Http2Tokio::accept(http2_buffer, http2_params, tcp_stream.into_split()).await?;
        let (frame_reader, mut http2) = tuple;
        let _jh = tokio::spawn(frame_reader);
        loop {
          let mut http2_stream = match http2.stream(ReqResBuffer::default()).await? {
            Either::Left(_) => return wtx::Result::Ok(()),
            Either::Right(elem) => elem,
          };
          let _stream_jh = tokio::spawn(async move {
            let fun = async {
              let (local_rrb, is_open) = http2_stream.recv_req().await?;
              if !is_open {
                return Ok(());
              }
              let mut req = local_rrb.into_http2_request(http2_stream.method());
              req.rrd.clear();
              req.rrd.body.extend_from_slice(b"Hello")?;
              let res = req.into_response(StatusCode::Ok);
              if http2_stream.send_res(res).await?.is_none() {
                return Ok(());
              }
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
