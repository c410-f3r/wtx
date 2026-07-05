//! An HTTP/2 server framework showcasing nested routes, middlewares, manual streams, dynamic routes,
//! PostgreSQL connections and JSON deserialization/serialization.

extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use core::{fmt::Write, ops::ControlFlow};
use tokio::net::{TcpStream, tcp::OwnedWriteHalf};
use wtx::{
  database::{DbClient, Record},
  http::{
    ManualStream, Method, MsgBufferString, MsgDataMut, Request, Response, StatusCode,
    http2_server_framework::{
      Http2ServerFramework, HttpRouter, JsonReply, Middleware, Path, State, StateClean,
      VerbatimParams, get, json,
    },
  },
  http2::{Http2ErrorCode, ServerStream},
  misc::SecretContext,
  pool::{PostgresRM, SimplePool},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, ROOT_CA, SECRET_KEY, host_from_args};

type LocalPool = SimplePool<PostgresRM<wtx::Error, TcpStream, TlsModeVerified>>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let mut uri = *b"postgres://USER:PASSWORD@localhost/DB_NAME";
  let mut server = Http2ServerFramework::tokio(
    ChaCha20::from_getrandom()?,
    TlsConfig::from_keys_pem(
      TlsModeVerified::default(),
      PUBLIC_KEY.try_into()?,
      SECRET_KEY.try_into()?,
    )?,
  )?;
  let pool = LocalPool::new(
    4,
    PostgresRM::new(
      ChaCha20::from_crypto_rng(server.rng_mut())?,
      SecretContext::new(server.rng_mut())?,
      TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?,
      &mut uri,
    )?,
  );
  let router = HttpRouter::paths(wtx::paths!(
    ("/db/{id}", get(db)),
    ("/json", json(Method::Post, deserialization_and_serialization)),
    (
      "/say",
      HttpRouter::new(
        wtx::paths!(("/hello", get(hello)), ("/world", get(world))),
        CustomMiddleware,
      )?,
    ),
    ("/stream", get(stream)),
  ))?;
  server.set_data(pool).run(&host_from_args(), router).await
}

async fn deserialization_and_serialization(state: State<'_, LocalPool>) -> wtx::Result<JsonReply> {
  let deserialize_example: DeserializeExample = serde_json::from_slice(&state.req.msg_data.body)?;
  let serialize_example = SerializeExample {
    _baz: [u32::from(deserialize_example._bar / 2), u32::from(deserialize_example._bar % 2)],
  };
  state.req.msg_data.clear();
  serde_json::to_writer(&mut state.req.msg_data.body, &serialize_example)?;
  Ok(JsonReply::default())
}

async fn db(state: StateClean<'_, LocalPool>, Path(id): Path<u32>) -> wtx::Result<VerbatimParams> {
  let mut lock = state.data.get_with_unit().await?;
  let record = lock.execute_stmt_single("SELECT name FROM persons WHERE id = $1", (id,)).await?;
  let name = record.decode::<_, &str>(0)?;
  state.req.msg_data.body.write_fmt(format_args!("Person of id `{id}` has name `{name}`"))?;
  Ok(VerbatimParams(StatusCode::Ok))
}

async fn hello() -> &'static str {
  "hello"
}

async fn stream(
  mut manual_stream: ManualStream<LocalPool, ServerStream<OwnedWriteHalf, TlsModeVerified>>,
) -> wtx::Result<()> {
  manual_stream.stream.common().send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}

async fn world() -> &'static str {
  "world"
}

struct CustomMiddleware;

impl Middleware<LocalPool, wtx::Error> for CustomMiddleware {
  type Aux = ();

  fn aux(&self) -> Self::Aux {}

  async fn req(
    &self,
    _: &mut LocalPool,
    _: &mut Self::Aux,
    _: &mut Request<MsgBufferString>,
  ) -> wtx::Result<ControlFlow<StatusCode, ()>> {
    println!("Inspecting request");
    Ok(ControlFlow::Continue(()))
  }

  async fn res(
    &self,
    _: &mut LocalPool,
    _: &mut Self::Aux,
    _: Response<&mut MsgBufferString>,
  ) -> wtx::Result<ControlFlow<StatusCode, ()>> {
    println!("Inspecting response");
    Ok(ControlFlow::Continue(()))
  }
}

#[derive(serde::Deserialize)]
struct DeserializeExample {
  _foo: u16,
  _bar: u16,
}

#[derive(serde::Serialize)]
struct SerializeExample {
  _baz: [u32; 2],
}
