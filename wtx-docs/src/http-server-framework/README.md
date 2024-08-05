# HTTP Server Framework

A small and fast to compile framework that can interact with many built-in features like PostgreSQL connections.

The bellow snippet requires ~30 dependencies and has an optimized binary size of ~690K.

```rust,edition2021,no_run
extern crate tokio;
extern crate wtx;

use wtx::http::{
  server_framework::{get, Router, ServerFramework},
  ReqResBuffer, Request, Response, StatusCode,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::paths(wtx::paths!(("hello-world", get(hello_world))));
  ServerFramework::new(router).listen("0.0.0.0:9000").await
}

async fn hello_world(mut req: Request<ReqResBuffer>) -> wtx::Result<Response<ReqResBuffer>> {
  req.rrd.clear();
  req.rrd.extend_body(b"Hello World")?;
  Ok(req.into_response(StatusCode::Ok))
}
```