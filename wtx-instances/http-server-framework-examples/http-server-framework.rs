//! An HTTP server framework showcasing nested routes, middlewares, manual streams, dynamic routes,
//! PostgreSQL connections and JSON deserialization/serialization.
//!
//! Currently, only HTTP/2 is supported.
//!
//! This snippet requires ~50 dependencies and has an optimized binary size of ~900K.

extern crate serde;
extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use core::{fmt::Write, ops::ControlFlow};
use tokio::net::{TcpStream, tcp::OwnedWriteHalf};
use wtx::{
  database::{Executor, Record},
  http::{
    ManualStream, ReqResBuffer, Request, Response, StatusCode,
    server_framework::{
      Middleware, PathOwned, Router, SerdeJson, ServerFrameworkBuilder, StateClean, get, post,
    },
  },
  http2::{Http2Buffer, Http2DataTokio, Http2ErrorCode, ServerStream},
  misc::{Xorshift64, simple_seed},
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
      Router::new(wtx::paths!(("/hello", get(hello)), ("/world", get(world))), CustomMiddleware,)?,
    ),
    ("/stream", get(stream)),
  ))?;
  let rm = PostgresRM::tokio("postgres://USER:PASSWORD@localhost/DB_NAME".into());
  let pool = Pool::new(4, rm);
  ServerFrameworkBuilder::new(router)
    .with_req_aux(move || pool.clone())
    .tokio(
      &wtx_instances::host_from_args(),
      Xorshift64::from(simple_seed()),
      |error| eprintln!("{error:?}"),
      |_| Ok(()),
    )
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

async fn stream(
  mut manual_stream: ManualStream<
    (),
    ServerStream<Http2DataTokio<Http2Buffer, OwnedWriteHalf, false>>,
    Pool,
  >,
) -> wtx::Result<()> {
  manual_stream.stream.common().send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}

async fn world() -> &'static str {
  "world"
}

struct CustomMiddleware;

impl Middleware<(), wtx::Error, Pool> for CustomMiddleware {
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {
    ()
  }

  async fn req(
    &self,
    _: &mut (),
    _: &mut Self::Aux,
    _: &mut Request<ReqResBuffer>,
    _: &mut Pool,
  ) -> wtx::Result<ControlFlow<StatusCode, ()>> {
    println!("Inspecting request");
    Ok(ControlFlow::Continue(()))
  }

  async fn res(
    &self,
    _: &mut (),
    _: &mut Self::Aux,
    _: Response<&mut ReqResBuffer>,
    _: &mut Pool,
  ) -> wtx::Result<ControlFlow<StatusCode, ()>> {
    println!("Inspecting response");
    Ok(ControlFlow::Continue(()))
  }
}
