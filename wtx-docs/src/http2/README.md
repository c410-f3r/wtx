# HTTP/2

Implementation of [RFC7541](https://datatracker.ietf.org/doc/html/rfc7541) and [RFC9113](https://datatracker.ietf.org/doc/html/rfc9113). In other words, a low-level HTTP.

Passes the `hpack-test-case` and the `h2spec` test suites. Due to official deprecation, server push and prioritization are not supported.

Activation feature is called `http2`.

```rust,edition2021
extern crate tokio;
extern crate wtx;

use wtx::{
  http::{Method, Request, ReqResBuffer},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{from_utf8_basic, UriRef},
  rng::NoStdRng,
};
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;

async fn client() {
  let uri = UriRef::new("127.0.0.1:9000");
  let mut http2 = Http2Tokio::connect(
    Http2Buffer::new(NoStdRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host().to_socket_addrs().unwrap().next().unwrap()).await.unwrap(),
  )
  .await
  .unwrap();
  let mut rrb = ReqResBuffer::default();
  let mut stream = http2.stream().await.unwrap();
  stream
    .send_req(Request::http2(Method::Get, b"Hello!"), &uri)
    .await
    .unwrap();
  let res = stream.recv_res(rrb).await.unwrap();
  println!("{}", from_utf8_basic(res.0.body()).unwrap());
  http2.send_go_away(Http2ErrorCode::NoError).await;
}
```