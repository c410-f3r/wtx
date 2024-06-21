# HTTP/2

Provides low and high level abstractions to interact with clients and servers.

```rust,edition2021
extern crate tokio;
extern crate wtx;

use wtx::{
  http::{Method, RequestStr},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio, StreamBuffer},
  misc::{from_utf8_basic, UriRef},
  rng::StaticRng,
};
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;

async fn client() {
  let uri = UriRef::new("127.0.0.1:9000");
  let mut http2 = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host().to_socket_addrs().unwrap().next().unwrap()).await.unwrap(),
  )
  .await
  .unwrap();
  let mut sb = StreamBuffer::default();
  let mut stream = http2.stream().await.unwrap();
  stream
    .send_req(&mut sb.hpack_enc_buffer, RequestStr::http2(b"Hello!", Method::Get, uri))
    .await
    .unwrap();
  let res = stream.recv_res(sb).await.unwrap();
  println!("{}", from_utf8_basic(&res.0.rrb.body).unwrap());
  http2.send_go_away(Http2ErrorCode::NoError).await;
}
```