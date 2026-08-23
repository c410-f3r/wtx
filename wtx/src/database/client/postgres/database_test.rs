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
  rng::{ChaCha20, Rng as _, SeedableRng as _, Xorshift64},
  tls::{TlsConfig, TlsConnectorBuilder, TlsCtx},
};
use alloc::string::String;

const BATCH_SIZE: usize = 8;
const MAX_STMTS: usize = 16;

/// Used in testing environments by the `db` macro.
//
// FIXME(STABLE): Use `from_std_random` instead of an insecure seed. In the meanwhile such a
//               case shouldn't be a serious problem for tests
#[doc(hidden)]
#[inline]
pub fn database_test<ER, EX, FUT, TCX, TS>(
  migration_dir: Option<&'static str>,
  tls_config: &TlsConfig<TCX>,
  cb: impl FnOnce(PostgresClient<ER, TS, TCX>) -> FUT,
) -> Result<FUT::Output, ER>
where
  ER: From<crate::Error>,
  EX: Executor<TcpStream = TS>,
  FUT: Future,
  TCX: TlsCtx,
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
    let mut rng = ChaCha20::from_key(Xorshift64::from_simple_seed()?.u8_32());
    {
      let mut client = PostgresClient::<_, _, _>::connect(
        ClientBuffer::new(MAX_STMTS, &mut rng),
        &Config::from_uri(&uri)?,
        TlsConnectorBuilder::new(EX::default(), uri).build(tls_config, &mut rng).await?,
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
        TlsConnectorBuilder::new(EX::default(), uri).build(tls_config, &mut rng).await?,
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
        TlsConnectorBuilder::new(EX::default(), uri).build(tls_config, &mut rng).await?,
      )
      .await?;
      let mut drop_db_query = String::new();
      drop_db_query.push_str("DROP DATABASE ");
      drop_db_query.push_str(&db_name);
      drop_db_query.push_str(" WITH (FORCE)");
      client.execute_ignored(drop_db_query.as_str()).await?;
    }

    Ok(test_result)
  })
}
