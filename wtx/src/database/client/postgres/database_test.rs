use crate::{
  calendar::timestamp_str,
  database::{
    DatabaseUriFromVars, DbClient as _,
    client::postgres::{ClientBuffer, Config, PostgresClient},
    schema_manager::Commands,
  },
  executor::{Executor, Runtime as _},
  misc::EnvVars,
  net::TcpStream,
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsConnectorBuilder, TlsModePlainText},
};
use alloc::string::String;

const BATCH_SIZE: usize = 8;
const MAX_STMTS: usize = 16;

/// Used in testing environments by the `db` macro.
#[doc(hidden)]
#[inline]
pub fn database_test<ER, EX, FUT, TS>(
  migration_dir: Option<&'static str>,
  cb: impl FnOnce(PostgresClient<ER, TS, TlsModePlainText>) -> FUT,
) -> Result<FUT::Output, ER>
where
  ER: From<crate::Error>,
  EX: Executor<TcpStream = TS>,
  FUT: Future,
  TS: TcpStream<Executor = EX>,
{
  EX::LocalRuntime::new()?.block_on(async move {
    let local_vars: DatabaseUriFromVars = EnvVars::from_available([])?.finish();
    let uri = local_vars.uri.as_str().into();

    let mut config = Config::from_uri(&uri)?;
    let mut db_name = String::new();
    db_name.push('_');
    db_name.push_str(timestamp_str(|dur| dur.as_nanos())?.1.as_str());

    let orig_db = String::from(config.db());
    let mut rng = ChaCha20::from_getrandom()?;
    let tls_config = TlsConfig::plaintext();

    {
      let mut client = PostgresClient::<_, _, _>::connect(
        ClientBuffer::new(MAX_STMTS, &mut rng),
        &Config::from_uri(&uri)?,
        TlsConnectorBuilder::new(EX::default(), uri).build(&tls_config, &mut rng).await?,
      )
      .await?;
      let mut create_db_query = String::new();
      create_db_query.push_str("CREATE DATABASE ");
      create_db_query.push_str(&db_name);
      client.execute_ignored(create_db_query.as_str()).await?;
    }

    let test_result = {
      config.set_db(db_name.as_str());
      let mut client = PostgresClient::<ER, _, _>::connect(
        ClientBuffer::new(MAX_STMTS, &mut rng),
        &config,
        TlsConnectorBuilder::new(EX::default(), uri).build(&tls_config, &mut rng).await?,
      )
      .await?;
      Commands::new(BATCH_SIZE, &mut client).clear_migrate_and_seed(migration_dir).await?;
      cb(client).await
    };

    {
      config.set_db(orig_db.as_str());
      let mut client = PostgresClient::<_, _, _>::connect(
        ClientBuffer::new(MAX_STMTS, &mut rng),
        &config,
        TlsConnectorBuilder::new(EX::default(), uri).build(&tls_config, &mut rng).await?,
      )
      .await?;
      let mut drop_db_query = String::new();
      drop_db_query.push_str("DROP DATABASE ");
      drop_db_query.push_str(&db_name);
      client.execute_ignored(drop_db_query.as_str()).await?;
    }

    Ok(test_result)
  })
}
