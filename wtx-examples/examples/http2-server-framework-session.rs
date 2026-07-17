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

use wtx::{
  collections::Vector,
  database::{DbClient, Record},
  executor::TokioExecutor,
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

type DbPool = SimplePool<PostgresRM<wtx::Error, TokioExecutor, TlsModeVerified>>;
type LocalSessionManager = SessionManager<u32, wtx::Error>;

fn main() -> wtx::Result<()> {
  let mut uri = *b"postgres://USER:PASSWORD@localhost/DB_NAME";
  let mut rng = ChaCha20::from_getrandom()?;
  let secret_context = SecretContext::new(&mut rng)?;
  let db_pool = DbPool::new(
    4,
    PostgresRM::tokio(
      ChaCha20::from_crypto_rng(&mut rng)?,
      secret_context.clone(),
      TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?,
      &mut uri,
    )?,
  );
  let (cleaner, session_manager) = LocalSessionManager::builder().build_generating_key(
    &mut rng,
    secret_context.clone(),
    db_pool.clone(),
  )?;
  tokio::spawn(async move {
    if let Err(err) = cleaner.await {
      eprintln!("{err}");
    }
  });
  let tls_config = TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    PUBLIC_KEY.try_into()?,
    &mut rng,
    (secret_context, &mut SECRET_KEY.clone()),
  )?;
  let router = HttpRouter::new(
    wtx::paths!(("/login", post(login)), ("/logout", get(logout))),
    SessionMiddleware::new(Vector::new(), session_manager.clone(), db_pool.clone()),
  )?;
  Http2ServerFramework::new(TokioExecutor::default(), rng, tls_config)?
    .set_data(Data { db_pool, session_manager, session_state: None })
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
}

async fn login(state: State<'_, Data>) -> wtx::Result<DynParams> {
  let Data { db_pool, session_manager, session_state } = state.data;
  if session_state.is_some() {
    state.req.clear();
    session_manager.delete_session_cookie(&mut state.req.msg_data, session_state, db_pool).await?;
    return Ok(DynParams::Verbatim(StatusCode::Forbidden));
  }
  let user: UserLoginReq<'_> = serde_json::from_slice(state.req.msg_data.body())?;
  let mut pool_guard = db_pool.get_with_unit().await?;
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
    .set_session_cookie(id, &mut state.req.msg_data, &mut ChaCha20::from_getrandom()?, db_pool)
    .await?;
  Ok(DynParams::Verbatim(StatusCode::Ok))
}

async fn logout(state: StateClean<'_, Data>) -> wtx::Result<StatusCode> {
  let Data { db_pool, session_manager, session_state } = state.data;
  if session_state.is_some() {
    session_manager.delete_session_cookie(&mut state.req.msg_data, session_state, db_pool).await?;
    Ok(StatusCode::Ok)
  } else {
    Ok(StatusCode::Forbidden)
  }
}

#[derive(Clone, Debug, wtx::Lease)]
struct Data {
  db_pool: DbPool,
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
