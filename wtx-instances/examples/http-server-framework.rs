//! An HTTP server framework showcasing nested routes, request middlewares, response
//! middlewares, dynamic routes, PostgreSQL connections and JSON deserialization/serialization.

use core::fmt::Write;
use std::sync::LazyLock;
use tokio::net::TcpStream;
use wtx::{
  database::{Executor, Record},
  http::{
    server_framework::{get, post, Router, ServerFramework},
    ReqResBuffer, Request, Response, StatusCode,
  },
  paths,
  pool::{Pool, PostgresRM, SimplePoolTokio},
};

static POOL: LazyLock<SimplePoolTokio<PostgresRM<wtx::Error, TcpStream>>> = LazyLock::new(|| {
  SimplePoolTokio::new(4, PostgresRM::tokio("postgres://USER:PASSWORD@localhost:5432/DB_NAME"))
});

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::paths(paths!(
    ("db/{id}", get(db)),
    ("json", post(json)),
    (
      "say",
      Router::new(
        paths!(("hello", get(hello)), ("world", get(world))),
        (request_middleware,),
        (response_middleware,),
      ),
    ),
  ));
  ServerFramework::new(router).listen(&wtx_instances::host_from_args()).await
}

async fn db((id, mut req): (u32, Request<ReqResBuffer>)) -> wtx::Result<Response<ReqResBuffer>> {
  let mut lock = POOL.get(&(), &()).await?;
  let record = lock.fetch_with_stmt("SELECT name FROM persons WHERE id = $1", (id,)).await?;
  let name = record.decode::<_, &str>(0)?;
  req.rrd.clear();
  req.rrd.write_fmt(format_args!("Person of id `1` has name `{name}`"))?;
  Ok(req.into_response(StatusCode::Ok))
}

async fn hello(mut req: Request<ReqResBuffer>) -> wtx::Result<Response<ReqResBuffer>> {
  req.rrd.clear();
  req.rrd.extend_body(b"hello")?;
  Ok(req.into_response(StatusCode::Ok))
}

async fn json(mut req: Request<ReqResBuffer>) -> wtx::Result<Response<ReqResBuffer>> {
  #[derive(serde::Deserialize)]
  struct DeserializeExample<'str> {
    _foo: i32,
    _bar: &'str str,
  }

  #[derive(serde::Serialize)]
  struct SerializeExample<'bytes> {
    _baz: &'bytes [u8],
  }

  let _de: DeserializeExample<'_> = simd_json::from_slice(req.rrd.body_mut())?;
  req.rrd.clear();
  simd_json::to_writer(&mut req.rrd, &SerializeExample { _baz: &[1, 2, 3, 4, 5] })?;
  Ok(req.into_response(StatusCode::Ok))
}

async fn request_middleware(_: &mut Request<ReqResBuffer>) -> wtx::Result<()> {
  println!("Before response");
  Ok(())
}

async fn response_middleware(_: &mut Response<ReqResBuffer>) -> wtx::Result<()> {
  println!("After response");
  Ok(())
}

async fn world(mut req: Request<ReqResBuffer>) -> wtx::Result<Response<ReqResBuffer>> {
  req.rrd.clear();
  req.rrd.extend_body(b"world")?;
  Ok(req.into_response(StatusCode::Ok))
}
