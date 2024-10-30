//! An HTTP server framework showcasing nested routes, request middlewares, response
//! middlewares, dynamic routes, PostgreSQL connections and JSON deserialization/serialization.
//!
//! Currently, only HTTP/2 is supported.
//!
//! This snippet requires ~50 dependencies and has an optimized binary size of ~900K.

extern crate serde;
extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use core::fmt::Write;
use tokio::net::TcpStream;
use wtx::{
  database::{Executor, Record},
  http::{
    server_framework::{
      get, post, PathOwned, Router, SerdeJson, ServerFrameworkBuilder, StateClean,
    },
    ReqResBuffer, Request, Response, StatusCode,
  },
  misc::{simple_seed, FnFutWrapper, Xorshift64},
  pool::{PostgresRM, SimplePoolTokio},
};

type Pool = SimplePoolTokio<PostgresRM<wtx::Error, TcpStream>>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::paths(wtx::paths!(
    ("/db/{id}", get(db)),
    ("/json", post(json)),
    (
      "/say",
      Router::new(
        wtx::paths!(("/hello", get(hello)), ("/world", get(world))),
        FnFutWrapper::from(request_middleware),
        FnFutWrapper::from(response_middleware),
      )?,
    ),
  ))?;
  let rm = PostgresRM::tokio("postgres://USER:PASSWORD@localhost/DB_NAME".into());
  let pool = Pool::new(4, rm);
  ServerFrameworkBuilder::new(router)
    .with_req_aux(move || pool.clone())
    .listen_tokio(&wtx_instances::host_from_args(), Xorshift64::from(simple_seed()), |error| {
      eprintln!("{error:?}")
    })
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

async fn db(
  state: StateClean<'_, (), Pool, ReqResBuffer>,
  PathOwned(id): PathOwned<u32>,
) -> wtx::Result<StatusCode> {
  let mut lock = state.stream_aux.get().await?;
  let record = lock.fetch_with_stmt("SELECT name FROM persons WHERE id = $1", (id,)).await?;
  let name = record.decode::<_, &str>(0)?;
  state.req.rrd.body.write_fmt(format_args!("Person of id `1` has name `{name}`"))?;
  Ok(StatusCode::Ok)
}

async fn hello() -> &'static str {
  "hello"
}

async fn json(_: SerdeJson<DeserializeExample>) -> wtx::Result<SerdeJson<SerializeExample>> {
  Ok(SerdeJson(SerializeExample { _baz: [1, 2, 3, 4] }))
}

async fn request_middleware(
  _: &mut (),
  _: &mut Pool,
  _: &mut Request<ReqResBuffer>,
) -> wtx::Result<()> {
  println!("Before response");
  Ok(())
}

async fn response_middleware(
  _: &mut (),
  _: &mut Pool,
  _: Response<&mut ReqResBuffer>,
) -> wtx::Result<()> {
  println!("After response");
  Ok(())
}

async fn world() -> &'static str {
  "world"
}
