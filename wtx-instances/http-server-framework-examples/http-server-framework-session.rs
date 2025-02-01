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
//!   id BYTEA NOT NULL PRIMARY KEY,
//!   user_id INT NOT NULL,
//!   expires_at TIMESTAMPTZ NOT NULL
//! );
//! ALTER TABLE "session" ADD CONSTRAINT session__user__fk FOREIGN KEY (user_id) REFERENCES "user" (id);
//! ```

use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use tokio::net::TcpStream;
use wtx::{
  database::{Executor, Record},
  http::{
    server_framework::{get, post, Router, ServerFrameworkBuilder, State, StateClean},
    ReqResBuffer, ReqResData, SessionDecoder, SessionManagerTokio, SessionState, StatusCode,
  },
  misc::{argon2_pwd, Vector},
  pool::{PostgresRM, SimplePoolTokio},
};

type Pool = SimplePoolTokio<PostgresRM<wtx::Error, TcpStream>>;
type SessionManager = SessionManagerTokio<u32, wtx::Error>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = "postgres://USER:PASSWORD@localhost/DB_NAME";
  let pool = Pool::new(4, PostgresRM::tokio(uri.into()));
  let mut rng = ChaCha20Rng::try_from_os_rng()?;
  let (expired, sm) = SessionManager::builder().build_generating_key(&mut rng, pool.clone());
  let router = Router::new(
    wtx::paths!(("/login", post(login)), ("/logout", get(logout))),
    SessionDecoder::new(sm.clone(), pool.clone()),
  )?;
  tokio::spawn(async move {
    if let Err(err) = expired.await {
      eprintln!("{err}");
    }
  });
  let rng_clone = rng.clone();
  ServerFrameworkBuilder::new(router)
    .with_conn_aux(move || ConnAux {
      pool: pool.clone(),
      rng: rng_clone.clone(),
      session_manager: sm.clone(),
      session_state: None,
    })
    .tokio("0.0.0.0:9000", rng, |err| eprintln!("{err:?}"), |_| Ok(()))
    .await?;
  Ok(())
}

#[inline]
async fn login(state: State<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  let ConnAux { pool, rng, session_manager, session_state } = state.conn_aux;
  if session_state.is_some() {
    session_manager.delete_session_cookie(&mut state.req.rrd, session_state, pool).await?;
    return Ok(StatusCode::Forbidden);
  }
  let user: UserLoginReq<'_> = serde_json::from_slice(state.req.rrd.body())?;
  let mut pool_guard = pool.get().await?;
  let record = pool_guard
    .fetch_with_stmt("SELECT id,first_name,password,salt FROM user WHERE email = $1", (user.email,))
    .await?;
  let id = record.decode::<_, u32>(0)?;
  let first_name = record.decode::<_, &str>(1)?;
  let pw_db = record.decode::<_, &[u8]>(2)?;
  let salt = record.decode::<_, &str>(3)?;
  let pw_req = argon2_pwd::<32>(&mut Vector::new(), user.password.as_bytes(), salt.as_bytes())?;
  state.req.rrd.clear();
  if pw_db != &pw_req {
    return Ok(StatusCode::Unauthorized);
  }
  serde_json::to_writer(&mut state.req.rrd.body, &UserLoginRes { id, name: first_name })?;
  drop(pool_guard);
  session_manager.set_session_cookie(id, rng, &mut state.req.rrd, pool).await?;
  Ok(StatusCode::Ok)
}

#[inline]
async fn logout(state: StateClean<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  let ConnAux { pool, rng: _, session_manager, session_state } = state.conn_aux;
  if session_state.is_some() {
    session_manager.delete_session_cookie(&mut state.req.rrd, session_state, pool).await?;
  }
  Ok(StatusCode::Ok)
}

#[derive(Clone, Debug, wtx_macros::ConnAux)]
struct ConnAux {
  pool: Pool,
  rng: ChaCha20Rng,
  session_manager: SessionManager,
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
