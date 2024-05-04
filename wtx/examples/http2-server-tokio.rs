//! Http2 echo server.

#[path = "./common/mod.rs"]
mod common;

use wtx::{
  http::{server::TokioHttp2, Headers, Method, RequestMut, Response, StatusCode},
  misc::ByteVector,
};

#[tokio::main]
async fn main() {
  TokioHttp2::tokio_http2(
    common::_host_from_args().parse().unwrap(),
    None,
    |err| println!("Connection error: {err:?}"),
    |err| println!("Request error: {err:?}"),
    handle,
  )
  .await
  .unwrap()
}

async fn handle<'buffer>(
  req: RequestMut<'buffer, 'buffer, 'buffer, ByteVector>,
) -> Result<Response<(&'buffer mut ByteVector, &'buffer mut Headers)>, ()> {
  req.headers.clear();
  println!("{:?}", core::str::from_utf8(req.data));
  Ok(match (req.uri.path(), req.method) {
    ("/", Method::Get) => Response::http2((req.data, req.headers), StatusCode::Ok),
    _ => {
      req.data.clear();
      Response::http2((req.data, req.headers), StatusCode::NotFound)
    }
  })
}
