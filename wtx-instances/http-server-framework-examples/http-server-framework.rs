//! An HTTP server framework showcasing nested routes, middlewares, manual streams, dynamic routes,
//! PostgreSQL connections and JSON deserialization/serialization.
//!
//! Currently, only HTTP/2 is supported.

extern crate serde;
extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use core::{fmt::Write, ops::ControlFlow};
use tokio::net::{TcpStream, tcp::OwnedWriteHalf};
use wtx::{
  database::{Executor, Record},
  http::{
    ManualStream, Method, ReqResBuffer, Request, Response, StatusCode,
    server_framework::{
      Middleware, PathOwned, Router, SerdeJsonOwned, ServerFrameworkBuilder, StateClean,
      VerbatimParams, get, json,
    },
  },
  http2::{Http2Buffer, Http2ErrorCode, ServerStream},
  pool::{PostgresRM, SimplePool},
  rng::{ChaCha20, SeedableRng},
};

type LocalPool = SimplePool<PostgresRM<wtx::Error, TcpStream>>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::paths(wtx::paths!(
    ("/db/{id}", get(db)),
    ("/json", json(Method::Post, deserialization_and_serialization)),
    (
      "/say",
      Router::new(wtx::paths!(("/hello", get(hello)), ("/world", get(world))), CustomMiddleware,)?,
    ),
    ("/stream", get(stream)),
  ))?;
  let pool = LocalPool::new(
    4,
    PostgresRM::tokio(ChaCha20::from_os()?, "postgres://USER:PASSWORD@localhost/DB_NAME".into()),
  );
  ServerFrameworkBuilder::new(ChaCha20::from_os()?, router)
    .with_stream_aux(move |_| Ok(pool.clone()))
    .tokio(
      &wtx_instances::host_from_args(),
      |error| eprintln!("{error:?}"),
      |_| Ok(()),
      |error| eprintln!("{error:?}"),
    )
    .await
}

async fn deserialization_and_serialization(
  _: SerdeJsonOwned<DeserializeExample>,
) -> wtx::Result<SerdeJsonOwned<SerializeExample>> {
  Ok(SerdeJsonOwned(SerializeExample { _baz: [1, 2, 3, 4] }))
}

async fn db(
  state: StateClean<'_, (), LocalPool, ReqResBuffer>,
  PathOwned(id): PathOwned<u32>,
) -> wtx::Result<VerbatimParams> {
  let mut lock = state.stream_aux.get_with_unit().await?;
  let record = lock.execute_stmt_single("SELECT name FROM persons WHERE id = $1", (id,)).await?;
  let name = record.decode::<_, &str>(0)?;
  state.req.rrd.body.write_fmt(format_args!("Person of id `{id}` has name `{name}`"))?;
  Ok(VerbatimParams(StatusCode::Ok))
}

async fn hello() -> &'static str {
  "hello"
}

async fn stream(
  mut manual_stream: ManualStream<(), ServerStream<Http2Buffer, OwnedWriteHalf>, LocalPool>,
) -> wtx::Result<()> {
  manual_stream.stream.common().send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}

async fn world() -> &'static str {
  "world"
}

struct CustomMiddleware;

impl Middleware<(), wtx::Error, LocalPool> for CustomMiddleware {
  type Aux = ();

  fn aux(&self) -> Self::Aux {}

  async fn req(
    &self,
    _: &mut (),
    _: &mut Self::Aux,
    _: &mut Request<ReqResBuffer>,
    _: &mut LocalPool,
  ) -> wtx::Result<ControlFlow<StatusCode, ()>> {
    println!("Inspecting request");
    Ok(ControlFlow::Continue(()))
  }

  async fn res(
    &self,
    _: &mut (),
    _: &mut Self::Aux,
    _: Response<&mut ReqResBuffer>,
    _: &mut LocalPool,
  ) -> wtx::Result<ControlFlow<StatusCode, ()>> {
    println!("Inspecting response");
    Ok(ControlFlow::Continue(()))
  }
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
