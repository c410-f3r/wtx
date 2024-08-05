# HTTP Client Framework

High-level pool of HTTP clients that currently only supports HTTP/2. Allows multiple connections that can be referenced in concurrent scenarios.

Activation feature is called `http-client-framework`.

```rust,edition2021,no_run
extern crate wtx;

use wtx::{http::{Client, ReqBuilder}, misc::{Uri, from_utf8_basic}};

async fn get_and_print() -> wtx::Result<()> {
  let client = Client::tokio_rustls(1).build();
  let res = ReqBuilder::get().send(&client, &Uri::new("https:://www.dukcduckgo.com:443")).await?;
  println!("{}", from_utf8_basic(res.rrd.body()).unwrap());
  Ok(())
}
```