# HTTP/2

Provides low and high level abstractions to interact with clients and servers.

```ignore,rust,edition2021
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use wtx::{
  http::Method,
  http2::{ErrorCode, Http2Buffer, Http2Params, Http2Tokio, ReqResBuffer},
  misc::{from_utf8_basic, UriRef},
  rng::StaticRng,
};

#[tokio::main]
async fn main() {
  let mut rrb = ReqResBuffer::default();
  rrb.data.reserve(6);
  rrb.data.extend_from_slice(b"Hello!").unwrap();
  rrb.uri.push_str("127.0.0.1:9000").unwrap();
  let uri = UriRef::new(&rrb.uri);
  let mut http2 = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host().to_socket_addrs().unwrap().next().unwrap()).await.unwrap(),
  )
  .await
  .unwrap();
  let mut stream = http2.stream().await.unwrap();
  stream.send_req(rrb.as_http2_request_ref(Method::Get)).await.unwrap();
  let res = stream.recv_res(&mut rrb).await.unwrap();
  println!("{}", from_utf8_basic(res.resource().unwrap().body()).unwrap());
  http2.send_go_away(ErrorCode::NoError).await.unwrap();
}
```