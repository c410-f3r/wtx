//! h2load

use wtx::{
  executor::StdExecutor,
  http::http2_server_framework::{Http2ServerFramework, HttpRouter, State, get},
};

#[wtx::main]
async fn main() -> wtx::Result<()> {
  let router = HttpRouter::paths(wtx::paths!(("/", get(root)),))?;
  Http2ServerFramework::new(StdExecutor::default())?.run("127.0.0.1:9000", router).await
}

async fn root(_state: State<'_, ()>) -> wtx::Result<()> {
  Ok(())
}
