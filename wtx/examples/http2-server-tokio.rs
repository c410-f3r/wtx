//! Http2 server.

#[path = "./common/mod.rs"]
mod common;

use tokio::net::TcpListener;
use wtx::{
  http::{Method, Response, StatusCode},
  http2::{AcceptParams, Http2Buffer, Http2Tokio, ReqResBuffer},
  rng::StaticRng,
};

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  loop {
    let (tcp_stream, _) = listener.accept().await.unwrap();
    let _jh_conn = tokio::spawn(async move {
      let ap = AcceptParams::default();
      let mut http2 =
        Http2Tokio::accept(ap, Http2Buffer::with_capacity(StaticRng::default()), tcp_stream)
          .await
          .unwrap();
      loop {
        let mut rrb = ReqResBuffer::default();
        let Some(stream) = http2.recv_stream(&mut rrb.headers).await.unwrap() else {
          break;
        };
        let _jh_stream = tokio::spawn(async move {
          let req = stream.recv_req(&mut rrb).await.unwrap();
          req.headers.clear();
          let res = match (req.uri.path(), req.method) {
            ("/", Method::Get) => Response::http2(("Hello!", req.headers), StatusCode::Ok),
            _ => Response::http2(("", req.headers), StatusCode::NotFound),
          };
          stream.send_res(res).await.unwrap();
        });
      }
    });
  }
}
