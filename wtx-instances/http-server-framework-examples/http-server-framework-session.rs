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
//! ALTER TABLE "user" ADD CONSTRAINT user__email__uq UNIQUE (email);
//!
//! CREATE TABLE session (
//!   id BYTEA NOT NULL PRIMARY KEY,
//!   user_id INT NOT NULL,
//!   expires_at TIMESTAMPTZ NOT NULL
//! );
//! ALTER TABLE session ADD CONSTRAINT session__user__fk FOREIGN KEY (user_id) REFERENCES "user" (id);
//! ```

use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use tokio::net::TcpStream;
use wtx::{
  database::{Executor, Record},
  http::{
    server_framework::{get, post, Router, ServerFrameworkBuilder, State, StateClean},
    ReqResBuffer, ReqResData, SessionDecoder, SessionEnforcer, SessionTokio, StatusCode,
  },
  misc::argon2_pwd,
  pool::{PostgresRM, SimplePoolTokio},
};

type ConnAux = (Session, ChaCha20Rng);
type Pool = SimplePoolTokio<PostgresRM<wtx::Error, TcpStream>>;
type Session = SessionTokio<u32, wtx::Error, Pool>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(
    wtx::paths!(("/login", post(login)), ("/logout", get(logout)),),
    (SessionDecoder::new(), SessionEnforcer::new(["/admin"])),
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
    .listen_tokio("0.0.0.0:9000", rng, |err| eprintln!("{err:?}"))
    .await?;
  Ok(())
}

#[inline]
async fn login(state: State<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  let (session, rng) = state.ca;
  if session.content.lock().await.state().is_some() {
    session.delete_session_cookie(&mut state.req.rrd).await?;
    return Ok(StatusCode::Forbidden);
  }
  let user: UserLoginReq<'_> = serde_json::from_slice(state.req.rrd.body())?;
  let mut executor_guard = session.store.get().await?;
  let record = executor_guard
    .fetch_with_stmt("SELECT id,password,salt FROM user WHERE email = $1", (user.email,))
    .await?;
  let id = record.decode::<_, u32>(0)?;
  let password_db = record.decode::<_, &[u8]>(1)?;
  let salt = record.decode::<_, &[u8]>(2)?;
  let password_req = argon2_pwd(user.password.as_bytes(), salt)?;
  state.req.rrd.clear();
  if password_db != &password_req {
    return Ok(StatusCode::Unauthorized);
  }
  drop(executor_guard);
  session.set_session_cookie(id, rng, &mut state.req.rrd).await?;
  serde_json::to_writer(&mut state.req.rrd.body, &UserLoginRes { id })?;
  Ok(StatusCode::Ok)
}

#[inline]
async fn logout(state: StateClean<'_, ConnAux, (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  state.ca.0.delete_session_cookie(&mut state.req.rrd).await?;
  Ok(StatusCode::Ok)
}

#[derive(Debug, serde::Deserialize)]
struct UserLoginReq<'req> {
  email: &'req str,
  password: &'req str,
}

#[derive(Debug, serde::Serialize)]
struct UserLoginRes {
  id: u32,
}
