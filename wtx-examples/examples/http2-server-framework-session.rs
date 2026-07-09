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
//! CREATE TABLE _wtx.session (
//!   key VARCHAR(32) NOT NULL PRIMARY KEY,
//!   csrf VARCHAR(32) NOT NULL,
//!   custom_state INT NOT NULL,
//!   expires_at TIMESTAMPTZ NOT NULL
//! );
//! ALTER TABLE _wtx.session ADD CONSTRAINT session__user__fk FOREIGN KEY (custom_state) REFERENCES "user" (id);
//! ```

use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  database::{DbClient, Record},
  http::{
    MsgData, SessionManager, SessionMiddleware, SessionState, StatusCode,
    http2_server_framework::{
      DynParams, Http2ServerFramework, HttpRouter, State, StateClean, get, post,
    },
  },
  misc::{SecretContext, argon2_pwd},
  pool::{PostgresRM, SimplePool},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, ROOT_CA, SECRET_KEY, host_from_args};

type DbPool = SimplePool<PostgresRM<wtx::Error, TcpStream, TlsModeVerified>>;
type LocalSessionManager = SessionManager<u32, wtx::Error>;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let mut uri = *b"postgres://USER:PASSWORD@localhost/DB_NAME";
  let mut server = Http2ServerFramework::tokio(TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    PUBLIC_KEY.try_into()?,
    SECRET_KEY.try_into()?,
  )?)?;
  let secret_context = SecretContext::new(server.rng_mut())?;
  let pool = DbPool::new(
    4,
    PostgresRM::new(
      ChaCha20::from_crypto_rng(server.rng_mut())?,
      secret_context.clone(),
      TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?,
      &mut uri,
    )?,
  );
  let (cleaner, session_manager) = LocalSessionManager::builder().build_generating_key(
    server.rng_mut(),
    secret_context,
    pool.clone(),
  )?;
  tokio::spawn(async move {
    if let Err(err) = cleaner.await {
      eprintln!("{err}");
    }
  });
  let router = HttpRouter::new(
    wtx::paths!(("/login", post(login)), ("/logout", get(logout))),
    SessionMiddleware::new(Vector::new(), session_manager.clone(), pool.clone()),
  )?;
  server
    .set_data(Data { pool, session_manager, session_state: None })
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run(&host_from_args(), router)
    .await
}

async fn login(state: State<'_, Data>) -> wtx::Result<DynParams> {
  let Data { pool, session_manager, session_state } = state.data;
  if session_state.is_some() {
    state.req.clear();
    session_manager.delete_session_cookie(&mut state.req.msg_data, session_state, pool).await?;
    return Ok(DynParams::Verbatim(StatusCode::Forbidden));
  }
  let user: UserLoginReq<'_> = serde_json::from_slice(state.req.msg_data.body())?;
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
  serde_json::to_writer(&mut state.req.msg_data.body, &UserLoginRes { id, name: first_name })?;
  drop(pool_guard);
  session_manager
    .set_session_cookie(id, &mut state.req.msg_data, &mut ChaCha20::from_getrandom()?, pool)
    .await?;
  Ok(DynParams::Verbatim(StatusCode::Ok))
}

async fn logout(state: StateClean<'_, Data>) -> wtx::Result<StatusCode> {
  let Data { pool, session_manager, session_state } = state.data;
  if session_state.is_some() {
    session_manager.delete_session_cookie(&mut state.req.msg_data, session_state, pool).await?;
    Ok(StatusCode::Ok)
  } else {
    Ok(StatusCode::Forbidden)
  }
}

#[derive(Clone, Debug, wtx::Lease)]
struct Data {
  pool: DbPool,
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
