//! Sessions contain information that live until the end of a connection. Optionally, they can be
//! stored in an external store.
//!
//! This example illustrates an authentication system using Argon2 and PostgreSQL.
//!
//! ```sql
//! CREATE TABLE "user" (
//!   id INT NOT NULL PRIMARY KEY,
//!   email VARCHAR(128) NOT NULL,
//!   first_name VARCHAR(32) NOT NULL,
//!   password BYTEA NOT NULL,
//!   salt CHAR(32) NOT NULL
//! );
//! ALTER TABLE "user" ADD CONSTRAINT user__email__uq UNIQUE (email);
//!
//! CREATE TABLE "session" (
//!   key VARCHAR(32) NOT NULL PRIMARY KEY,
//!   csrf VARCHAR(32) NOT NULL,
//!   custom_state INT NOT NULL,
//!   expires_at TIMESTAMPTZ NOT NULL
//! );
//! ALTER TABLE "session" ADD CONSTRAINT session__user__fk FOREIGN KEY (custom_state) REFERENCES "user" (id);
//! ```

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  database::{Executor, Record},
  http::{
    ReqResBuffer, ReqResData, SessionManager, SessionMiddleware, SessionState, StatusCode,
    server_framework::{DynParams, Router, ServerFrameworkBuilder, State, StateClean, get, post},
  },
  misc::argon2_pwd,
  pool::{PostgresRM, SimplePool},
  rng::{ChaCha20, SeedableRng},
};

type DbPool = SimplePool<PostgresRM<wtx::Error, TcpStream>>;
type LocalSessionManager = SessionManager<u32, wtx::Error>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DB_NAME";
  let mut db_rng = ChaCha20::from_os()?;
  let mut server_rng = ChaCha20::from_rng(&mut db_rng)?;
  let db_pool = DbPool::new(4, PostgresRM::tokio(db_rng, uri.into()));
  let builder = LocalSessionManager::builder();
  let (expired_sessions, sm) = builder.build_generating_key(&mut server_rng, db_pool.clone())?;
  let router = Router::new(
    wtx::paths!(("/login", post(login)), ("/logout", get(logout))),
    SessionMiddleware::new(Vector::new(), sm.clone(), db_pool.clone()),
  )?;
  tokio::spawn(async move {
    if let Err(err) = expired_sessions.await {
      eprintln!("{err}");
    }
  });
  ServerFrameworkBuilder::new(server_rng, router)
    .with_conn_aux(move |local_rng| {
      Ok(ConnAux {
        pool: db_pool.clone(),
        rng: local_rng,
        session_manager: sm.clone(),
        session_state: None,
      })
    })
    .tokio(
      "0.0.0.0:9000",
      |err| eprintln!("{err:?}"),
      |_| Ok(()),
      |_| Ok(()),
      |err| eprintln!("{err:?}"),
    )
    .await?;
  Ok(())
}

async fn login(state: State<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<DynParams> {
  let ConnAux { pool, rng, session_manager, session_state } = state.conn_aux;
  if session_state.is_some() {
    state.req.clear();
    session_manager.delete_session_cookie(&mut state.req.rrd, session_state, pool).await?;
    return Ok(DynParams::Verbatim(StatusCode::Forbidden));
  }
  let user: UserLoginReq<'_> = serde_json::from_slice(state.req.rrd.body())?;
  let mut pool_guard = pool.get_with_unit().await?;
  let record = pool_guard
    .execute_stmt_single(
      "SELECT id,first_name,password,salt FROM user WHERE email = $1",
      (user.email,),
    )
    .await?;
  let id = record.decode::<_, u32>(0)?;
  let first_name = record.decode::<_, &str>(1)?;
  let pw_db = record.decode::<_, &[u8]>(2)?;
  let salt = record.decode::<_, &str>(3)?;
  let pw_req = argon2_pwd::<32>(&mut Vector::new(), user.password.as_bytes(), salt.as_bytes())?;
  state.req.clear();
  if pw_db != pw_req {
    return Ok(DynParams::ClearAll(StatusCode::Unauthorized));
  }
  serde_json::to_writer(&mut state.req.rrd.body, &UserLoginRes { id, name: first_name })?;
  drop(pool_guard);
  session_manager.set_session_cookie(id, rng, &mut state.req.rrd, pool).await?;
  Ok(DynParams::Verbatim(StatusCode::Ok))
}

async fn logout(state: StateClean<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  let ConnAux { pool, rng: _, session_manager, session_state } = state.conn_aux;
  if session_state.is_some() {
    session_manager.delete_session_cookie(&mut state.req.rrd, session_state, pool).await?;
  }
  Ok(StatusCode::Ok)
}

#[derive(Clone, Debug, wtx_macros::ConnAux)]
struct ConnAux {
  pool: DbPool,
  rng: ChaCha20,
  session_manager: LocalSessionManager,
  session_state: Option<SessionState<u32>>,
}

#[derive(Debug, serde::Deserialize)]
struct UserLoginReq<'req> {
  email: &'req str,
  password: &'req str,
}

#[derive(Debug, serde::Serialize)]
struct UserLoginRes<'se> {
  id: u32,
  name: &'se str,
}
