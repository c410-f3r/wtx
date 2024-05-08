//! Http2 echo server.

#[path = "./common/mod.rs"]
mod common;

use wtx::{
  http::{server::TokioHttp2, Headers, Method, RequestStr, Response, StatusCode},
  misc::{from_utf8_basic, ByteVector},
};

#[tokio::main]
async fn main() {
  TokioHttp2::tokio_http2(
    common::_host_from_args().parse().unwrap(),
    None,
    |err| eprintln!("Connection error: {err:?}"),
    |err| eprintln!("Request error: {err:?}"),
    handle,
  )
  .await
  .unwrap()
}

async fn handle<'buffer>(
  req: RequestStr<'buffer, (&'buffer mut ByteVector, &'buffer mut Headers)>,
) -> Result<Response<(&'buffer mut ByteVector, &'buffer mut Headers)>, ()> {
  req.data.1.clear();
  println!("{}", from_utf8_basic(req.body()).unwrap());
  Ok(match (req.uri.path(), req.method) {
    ("/", Method::Get) => Response::http2(req.data, StatusCode::Ok),
    _ => {
      req.data.1.clear();
      Response::http2(req.data, StatusCode::NotFound)
    }
  })
}
