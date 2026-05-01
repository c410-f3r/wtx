use crate::{
  calendar::timestamp_str,
  database::{
    DatabaseUriFromVars, Executor as _,
    client::postgres::{Config, ExecutorBuffer, PostgresExecutor},
    schema_manager::Commands,
  },
  executor::Runtime,
  misc::EnvVars,
  rng::{ChaCha20, CryptoSeedableRng as _},
  sync::Arc,
};
use core::future::Future;
use std::{net::TcpStream, string::String};

const BATCH_SIZE: usize = 8;
const MAX_STMTS: usize = 16;

/// Used in testing environments by the `db` macro.
#[doc(hidden)]
pub async fn database_test<FUT>(
  migration_dir: Option<&'static str>,
  runtime: Arc<Runtime>,
  cb: impl FnOnce(PostgresExecutor<crate::Error, ExecutorBuffer, TcpStream>, Arc<Runtime>) -> FUT,
) -> crate::Result<FUT::Output>
where
  FUT: Future,
{
  let local_vars: DatabaseUriFromVars = EnvVars::from_available([])?.finish();
  let uri = local_vars.uri.as_str().into();

  let mut config = Config::from_uri(&uri)?;
  let mut db_name = String::new();
  db_name.push('_');
  db_name.push_str(timestamp_str(|dur| dur.as_nanos())?.1.as_str());

  let orig_db = String::from(config.db());
  let mut rng = ChaCha20::from_getrandom()?;

  {
    let mut conn = PostgresExecutor::<crate::Error, _, _>::connect(
      &Config::from_uri(&uri)?,
      ExecutorBuffer::new(MAX_STMTS, &mut rng),
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port())?,
    )
    .await?;
    let mut create_db_query = String::new();
    create_db_query.push_str("CREATE DATABASE ");
    create_db_query.push_str(&db_name);
    conn.execute_ignored(create_db_query.as_str()).await?;
  }

  let test_result = {
    config.set_db(db_name.as_str());
    let mut conn = PostgresExecutor::connect(
      &config,
      ExecutorBuffer::new(MAX_STMTS, &mut rng),
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port())?,
    )
    .await?;
    Commands::new(BATCH_SIZE, &mut conn).clear_migrate_and_seed(migration_dir).await?;
    cb(conn, runtime).await
  };

  {
    config.set_db(orig_db.as_str());
    let mut conn = PostgresExecutor::<crate::Error, _, _>::connect(
      &config,
      ExecutorBuffer::new(MAX_STMTS, &mut rng),
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port())?,
    )
    .await?;
    let mut drop_db_query = String::new();
    drop_db_query.push_str("DROP DATABASE ");
    drop_db_query.push_str(&db_name);
    conn.execute_ignored(drop_db_query.as_str()).await?;
  }

  Ok(test_result)
}
