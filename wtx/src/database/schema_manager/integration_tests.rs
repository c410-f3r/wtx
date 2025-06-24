mod backend;
mod db;
mod generic;
mod schema;

use crate::{
  collection::Vector,
  database::{
    Executor, Identifier,
    schema_manager::{
      Commands, DbMigration, MigrationStatus, SchemaManagement, UserMigrationGroup,
      doc_tests::{user_migration, user_migration_group},
    },
  },
  de::DEController,
};
use alloc::string::String;
use core::fmt::{Debug, Write};
use tokio::net::TcpStream;

macro_rules! create_integration_test {
  ($executor:expr, $buffer:expr, $aux:expr, $($fun:path),*) => {{
    $({
      let (_buffer_cmd, _, _buffer_idents, _) = $buffer;
      let mut commands = crate::database::schema_manager::Commands::with_executor($executor);
      commands.clear((_buffer_cmd, _buffer_idents)).await.unwrap();
      $fun($buffer, &mut commands, $aux).await;
    })*
  }};
}

macro_rules! create_integration_tests {
  (
    $fn_name:ident,
    mysql: $($mysql:path),*;
    postgres: $($postgres:path),*;
  ) => {
    pub(crate) async fn $fn_name() {
      let mut _buffer_cmd = String::new();
      let mut _buffer_db_migrations = Vector::<DbMigration>::new();
      let mut _buffer_idents = Vector::<Identifier>::new();
      let mut _buffer_status = Vector::<MigrationStatus>::new();

      #[cfg(feature = "mysql")]
      create_integration_test!(
        {
          use crate::rng::SeedableRng;
          let uri_string = std::env::var("DATABASE_URI_MYSQL").unwrap();
          let uri = crate::misc::UriRef::new(&uri_string);
          let config = crate::database::client::mysql::Config::from_uri(&uri).unwrap();
          let stream = TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap();
          let mut rng = crate::rng::ChaCha20::from_seed(crate::tests::_32_bytes_seed()).unwrap();
          crate::database::client::mysql::MysqlExecutor::connect(
            &config,
            crate::database::client::mysql::ExecutorBuffer::new(usize::MAX, &mut rng),
            &mut rng,
            stream,
          ).await.unwrap()
        },
        (&mut _buffer_cmd, &mut _buffer_db_migrations, &mut _buffer_idents, &mut _buffer_status),
        _generic_schema(),
        $($mysql),*
      );

      #[cfg(feature = "postgres")]
      create_integration_test!(
        {
          use crate::rng::SeedableRng;
          let uri_string = std::env::var("DATABASE_URI_POSTGRES").unwrap();
          let uri = crate::misc::UriRef::new(&uri_string);
          let config = crate::database::client::postgres::Config::from_uri(&uri).unwrap();
          let stream = TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap();
          let mut rng = crate::rng::ChaCha20::from_seed(crate::tests::_32_bytes_seed()).unwrap();
          crate::database::client::postgres::PostgresExecutor::connect(
            &config,
            crate::database::client::postgres::ExecutorBuffer::new(usize::MAX, &mut rng),
            &mut rng,
            stream,
          ).await.unwrap()
        },
        (&mut _buffer_cmd, &mut _buffer_db_migrations, &mut _buffer_idents, &mut _buffer_status),
        _pg_schema(),
        $($postgres),*
      );
    }
  };
}

macro_rules! create_all_integration_tests {
  (
    mysql: $($mysql:path),*;
    postgres: $($postgres:path),*;

    generic: $($fun:path),*;

    with_schema: $($with_schema:path),*;
    without_schema: $($without_schema:path),*;
  ) => {
    create_integration_tests!(
      integration_tests_db,
      mysql: $($mysql),*;
      postgres: $($postgres),*;
    );

    create_integration_tests!(
      integration_tests_generic,
      mysql: $($fun),*;
      postgres: $($fun),*;
    );

    create_integration_tests!(
      integration_tests_schema,
      mysql: $($without_schema),*;
      postgres: $($with_schema),*;
    );

    #[tokio::test]
    async fn integration_tests() {
      integration_tests_db().await;
      integration_tests_generic().await;
      integration_tests_schema().await;
    }
  };
}

create_all_integration_tests!(
  // Database

  mysql:
    db::mysql::clean_drops_all_objs;
  postgres:
    db::postgres::clean_drops_all_objs;

  // Generic

  generic:
    generic::all_tables_returns_the_number_of_tables_of_the_default_schema,
    generic::rollback_works;

  // Schema

  with_schema:
    schema::with_schema::all_tables_returns_the_number_of_tables_of_wtx_schema,
    schema::with_schema::migrate_works;

  without_schema:
    schema::without_schema::_migrate_works;
);

#[derive(Clone, Copy)]
pub(crate) struct AuxTestParams {
  pub(crate) default_schema: &'static str,
  pub(crate) schema_regulator: usize,
  pub(crate) wtx_schema: &'static str,
}

pub(crate) async fn create_foo_table<E>(
  buffer_cmd: &mut String,
  c: &mut Commands<E>,
  schema_prefix: &str,
) where
  E: SchemaManagement,
  <<E as Executor>::Database as DEController>::Error: Debug,
{
  buffer_cmd.write_fmt(format_args!("CREATE TABLE {schema_prefix}foo(id INT)")).unwrap();
  c.executor_mut().execute(buffer_cmd.as_str(), |_| Ok(())).await.unwrap();
  buffer_cmd.clear();
}

pub(crate) fn _generic_schema() -> AuxTestParams {
  AuxTestParams { default_schema: "", wtx_schema: "", schema_regulator: 2 }
}

pub(crate) async fn _migrate_doc_test<E>(
  (buffer_cmd, buffer_db_migrations, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
  ),
  c: &mut Commands<E>,
) -> UserMigrationGroup<&'static str>
where
  E: SchemaManagement,
{
  let mg = user_migration_group();
  let _s = c.migrate((buffer_cmd, buffer_db_migrations), &mg, [&user_migration()]).await.unwrap();
  mg
}

pub(crate) fn _pg_schema() -> AuxTestParams {
  AuxTestParams { default_schema: "public", wtx_schema: "_wtx", schema_regulator: 0 }
}
