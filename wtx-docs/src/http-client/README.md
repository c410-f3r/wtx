# HTTP Client

High-level pool of HTTP clients that currently only supports HTTP/2. Allows multiple connections that can be referenced in concurrent scenarios.

Activation feature is called `http-client`.

```rust,edition2021
extern crate wtx;

use wtx::{
  http::{Client, Method, ReqResBuffer, ReqUri},
  misc::{from_utf8_basic, TokioRustlsConnector},
};

pub(crate) async fn get_and_print() -> wtx::Result<()> {
  let client = Client::tokio(1, |uri| async move {
    Ok(
      TokioRustlsConnector::from_webpki_roots()
        .with_tcp_stream(uri.host(), uri.hostname())
        .await?,
    )
  });
  let mut rrb = ReqResBuffer::default();
  rrb.set_uri_from_str("https:://www.bing.com:443").unwrap();
  let res = client.send(Method::Get, rrb).await.unwrap();
  println!("{}", from_utf8_basic(res.rrd.body()).unwrap());
  Ok(())
}
```