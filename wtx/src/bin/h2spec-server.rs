//! h2spec

use wtx::{
  http::{server::OptionedServer, Headers, RequestStr, Response, StatusCode},
  http2::{Http2Buffer, Http2Params, StreamBuffer},
  misc::ByteVector,
  rng::StdRng,
};

#[tokio::main]
async fn main() {
  OptionedServer::tokio_http2(
    "127.0.0.1:9000".parse().unwrap(),
    Some(60),
    |err| eprintln!("Error: {err:?}"),
    handle,
    || Ok(Http2Buffer::new(StdRng::default())),
    || Http2Params::default(),
    || {
      let mut rslt = StreamBuffer::default();
      rslt.rrb.body.reserve(5);
      Ok(rslt)
    },
    (|| {}, |_| {}, |_, stream| async move { Ok(stream) }),
  )
  .await
  .unwrap()
}

async fn handle<'buffer>(
  req: RequestStr<'buffer, (&'buffer mut ByteVector, &'buffer mut Headers)>,
) -> Result<Response<(&'buffer mut ByteVector, &'buffer mut Headers)>, wtx::Error> {
  req.data.0.clear();
  req.data.0.extend_from_slice(b"Hello").unwrap();
  req.data.1.clear();
  Ok(Response::http2(req.data, StatusCode::Ok))
}
