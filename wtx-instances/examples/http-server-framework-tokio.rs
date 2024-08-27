//! An HTTP server framework showcasing nested routes, request middlewares, response
//! middlewares, dynamic routes, PostgreSQL connections and JSON deserialization/serialization.
//!
//! Currently, only HTTP/2 is supported.
//!
//! This snippet requires ~50 dependencies and has an optimized binary size of ~900K.

extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use core::fmt::Write;
use std::sync::LazyLock;
use tokio::net::TcpStream;
use wtx::{
  database::{Executor, Record},
  http::{
    server_framework::{get, post, PathOwned, Router, SerdeJson, ServerFramework},
    ReqResBuffer, Request, Response, StatusCode,
  },
  misc::Vector,
  pool::{Pool, PostgresRM, SimplePoolTokio},
};

static POOL: LazyLock<SimplePoolTokio<PostgresRM<wtx::Error, TcpStream>>> = LazyLock::new(|| {
  SimplePoolTokio::new(4, PostgresRM::tokio("postgres://USER:PASSWORD@localhost:5432/DB_NAME"))
});

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::paths(wtx::paths!(
    ("/db/:id", get(db)),
    ("/json", post(json)),
    (
      "/say",
      Router::new(
        wtx::paths!(("/hello", get(hello)), ("/world", get(world))),
        (request_middleware,),
        (response_middleware,),
      ),
    ),
  ));
  ServerFramework::new(router)
    .listen(&wtx_instances::host_from_args(), |error| eprintln!("{error:?}"))
    .await
}

#[derive(serde::Deserialize)]
struct DeserializeExample {
  _foo: i32,
  _bar: u64,
}

#[derive(serde::Serialize)]
struct SerializeExample {
  _baz: [u8; 4],
}

async fn db(vec: &mut Vector<u8>, PathOwned(id): PathOwned<u32>) -> wtx::Result<StatusCode> {
  let mut lock = POOL.get(&(), &()).await?;
  let record = lock.fetch_with_stmt("SELECT name FROM persons WHERE id = $1", (id,)).await?;
  let name = record.decode::<_, &str>(0)?;
  vec.clear();
  vec.write_fmt(format_args!("Person of id `1` has name `{name}`"))?;
  Ok(StatusCode::Ok)
}

async fn hello() -> &'static str {
  "hello"
}

async fn json(_: SerdeJson<DeserializeExample>) -> wtx::Result<SerdeJson<SerializeExample>> {
  Ok(SerdeJson(SerializeExample { _baz: [1, 2, 3, 4] }))
}

async fn request_middleware(_: &mut Request<ReqResBuffer>) -> wtx::Result<()> {
  println!("Before response");
  Ok(())
}

async fn response_middleware(_: &mut Response<&mut ReqResBuffer>) -> wtx::Result<()> {
  println!("After response");
  Ok(())
}

async fn world() -> &'static str {
  "world"
}
