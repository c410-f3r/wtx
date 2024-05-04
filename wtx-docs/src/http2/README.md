# HTTP/2

Provides low and high level abstractions to interact with clients and servers.

```ignore,rust,edition2021
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use wtx::{
  http::{Headers, Method, Request},
  http2::{Http2Buffer, Http2Params, Http2Tokio, ReqResBuffer},
  misc::{from_utf8_basic, UriString},
  rng::StaticRng,
};

#[tokio::main]
async fn main() {
  let uri = UriString::new("127.0.0.1:9000");
  let mut http2 = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host().to_socket_addrs().unwrap().next().unwrap()).await.unwrap(),
  )
  .await
  .unwrap();
  let mut rrb = ReqResBuffer::default();
  let mut stream = http2.stream().await.unwrap();
  let res = stream
    .send_req_recv_res(
      Request::http2(&"Hello!", &Headers::new(0), Method::Get, uri.to_ref()),
      &mut rrb,
    )
    .await
    .unwrap();
  println!("{}", from_utf8_basic(res.resource().unwrap().body()).unwrap())
}
```