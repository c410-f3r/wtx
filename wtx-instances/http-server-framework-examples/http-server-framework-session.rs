//! Sessions contain information that live until the end of a connection. Optionally, they can be
//! stored in an external store.
//!
//! This example illustrates an authentication system using Argon2 and PostgreSQL.
//!
//! ```sql
//! CREATE TABLE "user" (
//!   id INT NOT NULL PRIMARY KEY,
//!   email VARCHAR(128) NOT NULL,
//!   password BYTEA NOT NULL,
//!   salt BYTEA NOT NULL
//! );
//!
//! CREATE TABLE session (
//!   id BYTEA NOT NULL PRIMARY KEY,
//!   user_id INT NOT NULL,
//!   expires_at TIMESTAMPTZ NOT NULL
//! );
//!
//! ALTER TABLE session ADD CONSTRAINT session__user__fk FOREIGN KEY (user_id) REFERENCES "user" (id);
//! ```

use argon2::{Algorithm, Argon2, Block, Params, Version};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use tokio::net::TcpStream;
use wtx::{
  database::{Executor, Record},
  http::{
    server_framework::{get, post, Router, ServerFrameworkBuilder, State, StateClean},
    ReqResBuffer, ReqResData, SessionDecoder, SessionEnforcer, SessionTokio, StatusCode,
  },
  pool::{PostgresRM, SimplePoolTokio},
};

const ARGON2_OUTPUT_LEN: usize = 32;
const ARGON2_PARAMS: Params = {
  let Ok(elem) = Params::new(
    Params::DEFAULT_M_COST,
    Params::DEFAULT_T_COST,
    Params::DEFAULT_P_COST,
    Some(ARGON2_OUTPUT_LEN),
  ) else {
    panic!();
  };
  elem
};
const LOGIN: &str = "/login";
const LOGOUT: &str = "/logout";

type ConnAux = (Session, ChaCha20Rng);
type Pool = SimplePoolTokio<PostgresRM<wtx::Error, TcpStream>>;
type Session = SessionTokio<u32, wtx::Error, Pool>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(
    wtx::paths!((LOGIN, post(login)), (LOGOUT, get(logout)),),
    (SessionDecoder::new(), SessionEnforcer::new([LOGIN, LOGOUT])),
    (),
  )?;
  let pool = Pool::new(4, PostgresRM::tokio("postgres://USER:PASSWORD@localhost/DB_NAME".into()));
  let mut rng = ChaCha20Rng::from_entropy();
  let (expired_sessions, session) = Session::builder(pool).build_generating_key(&mut rng);
  tokio::spawn(async move {
    if let Err(err) = expired_sessions.await {
      eprintln!("{err}");
    }
  });
  let rng_clone = rng.clone();
  ServerFrameworkBuilder::new(router)
    .with_conn_aux(move || (session.clone(), rng_clone.clone()))
    .listen("0.0.0.0:9000", rng, |err| eprintln!("{err:?}"))
    .await?;
  Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct User<'req> {
  email: &'req str,
  password: &'req str,
}

#[inline]
async fn login(state: State<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  let (session, rng) = state.ca;
  if session.content.lock().await.state().is_some() {
    session.delete_session_cookie(&mut state.req.rrd).await?;
    return Ok(StatusCode::Forbidden);
  }
  let user: User<'_> = serde_json::from_slice(state.req.rrd.body())?;
  let mut executor_guard = session.store.get().await?;
  let record = executor_guard
    .fetch_with_stmt("SELECT id,password,salt FROM user WHERE email = $1", (user.email,))
    .await?;
  let id = record.decode::<_, u32>(0)?;
  let password_db = record.decode::<_, &[u8]>(1)?;
  let salt = record.decode::<_, &[u8]>(2)?;
  let mut password_req = [0; ARGON2_OUTPUT_LEN];
  Argon2::new(Algorithm::Argon2id, Version::V0x13, ARGON2_PARAMS).hash_password_into_with_memory(
    user.password.as_bytes(),
    salt,
    &mut password_req,
    &mut [Block::new(); ARGON2_PARAMS.block_count()],
  )?;
  state.req.rrd.clear();
  if password_db != &password_req {
    return Ok(StatusCode::Unauthorized);
  }
  drop(executor_guard);
  session.set_session_cookie(id, rng, &mut state.req.rrd).await?;
  Ok(StatusCode::Ok)
}

#[inline]
async fn logout(state: StateClean<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  state.ca.0.delete_session_cookie(&mut state.req.rrd).await?;
  Ok(StatusCode::Ok)
}
