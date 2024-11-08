mod backend;
mod db;
mod generic;
mod schema;

use crate::{
  database::{
    schema_manager::{
      doc_tests::{migration, migration_group},
      Commands, DbMigration, MigrationGroup, SchemaManagement,
    },
    Identifier, DEFAULT_URI_VAR,
  },
  misc::{Vector, Xorshift64},
};
use alloc::string::String;
use core::fmt::Write;
use tokio::net::TcpStream;

macro_rules! create_integration_test {
  ($executor:expr, $buffer:expr, $aux:expr, $($fun:path),*) => {{
    $({
      let (_buffer_cmd, _, _buffer_idents) = $buffer;
      let mut commands = crate::database::schema_manager::Commands::with_executor($executor);
      commands.clear((_buffer_cmd, _buffer_idents)).await.unwrap();
      $fun($buffer, &mut commands, $aux).await;
    })*
  }};
}

macro_rules! create_integration_tests {
  (
    $fn_name:ident,
    postgres: $($postgres:path),*;
  ) => {
    pub(crate) async fn $fn_name() {
      let mut _buffer_cmd = String::new();
      let mut _buffer_db_migrations = Vector::<DbMigration>::new();
      let mut _buffer_idents = Vector::<Identifier>::new();

      create_integration_test!(
        {
          let uri = std::env::var(DEFAULT_URI_VAR).unwrap();
          let uri = crate::misc::UriRef::new(&uri);
          let config = crate::database::client::postgres::Config::from_uri(&uri).unwrap();
          let stream = TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap();
          let mut rng = Xorshift64::from(crate::misc::simple_seed());
          crate::database::client::postgres::Executor::connect(
            &config,
            crate::database::client::postgres::ExecutorBuffer::new(usize::MAX, &mut rng),
            &mut rng,
            stream,
          ).await.unwrap()
        },
        (&mut _buffer_cmd, &mut _buffer_db_migrations, &mut _buffer_idents),
        _pg_schema(),
        $($postgres),*
      );
    }
  };
}

macro_rules! create_all_integration_tests {
  (
    postgres: $($postgres:path),*;

    generic: $($fun:path),*;

    with_schema: $($with_schema:path),*;
    without_schema: $($without_schema:path),*;
  ) => {
    create_integration_tests!(
      integration_tests_db,
      postgres: $($postgres),*;
    );

    create_integration_tests!(
      integration_tests_generic,
      postgres: $($fun),*;
    );

    create_integration_tests!(
      integration_tests_schema,
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

  postgres:
    db::postgres::_clean_drops_all_objs;

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
{
  buffer_cmd.write_fmt(format_args!("CREATE TABLE {}foo(id INT)", schema_prefix)).unwrap();
  c.executor.execute(buffer_cmd.as_str(), |_| {}).await.unwrap();
  buffer_cmd.clear();
}

#[inline]
pub(crate) fn _generic_schema() -> AuxTestParams {
  AuxTestParams { default_schema: "", wtx_schema: "", schema_regulator: 2 }
}

#[inline]
pub(crate) async fn _migrate_doc_test<E>(
  (buffer_cmd, buffer_db_migrations, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
  ),
  c: &mut Commands<E>,
) -> MigrationGroup<&'static str>
where
  E: SchemaManagement,
{
  let mg = migration_group();
  c.migrate((buffer_cmd, buffer_db_migrations), &mg, [migration()].iter()).await.unwrap();
  mg
}

#[inline]
pub(crate) fn _mssql_schema() -> AuxTestParams {
  AuxTestParams { default_schema: "dbo", wtx_schema: "_wtx", schema_regulator: 0 }
}

#[inline]
pub(crate) fn _pg_schema() -> AuxTestParams {
  AuxTestParams { default_schema: "public", wtx_schema: "_wtx", schema_regulator: 0 }
}
