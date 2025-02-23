// Many commands were retrieved from the flyway project (https://github.com/flyway) so credits to
// the authors.

macro_rules! _wtx_migration_columns {
  () => {
    "_wtx_migration_omg_version INT NOT NULL, \
    checksum VARCHAR(20) NOT NULL, \
    name VARCHAR(128) NOT NULL, \
    repeatability INTEGER NULL, \
    version INT NOT NULL, \
    CONSTRAINT _wtx_migration_unq UNIQUE (version, _wtx_migration_omg_version)"
  };
}

macro_rules! _wtx_migration_group_columns {
  () => {
    "version INT NOT NULL PRIMARY KEY, \
    name VARCHAR(128) NOT NULL"
  };
}

macro_rules! _serial_id {
  () => {
    "id SERIAL NOT NULL PRIMARY KEY,"
  };
}

#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
pub(crate) mod postgres;

use crate::{
  database::{
    Database, DatabaseTy, FromRecord,
    executor::Executor,
    schema_manager::{DbMigration, MigrationGroup, UserMigration},
  },
  misc::{DEController, Lease, Vector},
};
use alloc::string::String;
use core::fmt::Write;

#[inline]
pub(crate) async fn _delete_migrations<E, S>(
  buffer_cmd: &mut String,
  executor: &mut E,
  mg: &MigrationGroup<S>,
  schema_prefix: &str,
  version: i32,
) -> Result<(), <E::Database as DEController>::Error>
where
  E: Executor,
  S: Lease<str>,
{
  buffer_cmd.write_fmt(format_args!(
    "DELETE FROM {schema_prefix}_wtx_migration WHERE _wtx_migration_omg_version = {mg_version} AND version > {version}",
    mg_version = mg.version(),
  )).map_err(crate::Error::from)?;
  executor.execute(buffer_cmd.as_str(), |_| Ok(())).await?;
  buffer_cmd.clear();
  Ok(())
}

#[inline]
pub(crate) async fn _insert_migrations<'migration, DBS, E, I, S>(
  buffer_cmd: &mut String,
  executor: &mut E,
  mg: &MigrationGroup<S>,
  migrations: I,
  schema_prefix: &str,
) -> Result<(), <E::Database as DEController>::Error>
where
  DBS: Lease<[DatabaseTy]> + 'migration,
  E: Executor,
  I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
  S: Lease<str> + 'migration,
{
  buffer_cmd
    .write_fmt(format_args!(
      "INSERT INTO {schema_prefix}_wtx_migration_group (version, name)
    SELECT * FROM (SELECT {mg_version} AS version, '{mg_name}' AS name) AS tmp
    WHERE NOT EXISTS (
      SELECT 1 FROM {schema_prefix}_wtx_migration_group WHERE version = {mg_version}
    );",
      mg_name = mg.name(),
      mg_version = mg.version(),
    ))
    .map_err(Into::into)?;
  executor.execute(buffer_cmd.as_str(), |_| Ok(())).await?;
  buffer_cmd.clear();

  for migration in migrations.clone() {
    buffer_cmd.push_str(migration.sql_up());
  }
  executor
    .transaction(|this| async {
      this.execute(buffer_cmd.as_str(), |_| Ok(())).await?;
      Ok(((), this))
    })
    .await?;
  buffer_cmd.clear();

  for migration in migrations {
    buffer_cmd
      .write_fmt(format_args!(
        "INSERT INTO {schema_prefix}_wtx_migration (
        version, _wtx_migration_omg_version, checksum, name
      ) VALUES (
        {m_version}, {mg_version}, '{m_checksum}', '{m_name}'
      );",
        m_checksum = migration.checksum(),
        m_name = migration.name(),
        m_version = migration.version(),
        mg_version = mg.version(),
        schema_prefix = schema_prefix,
      ))
      .map_err(Into::into)?;
  }
  executor
    .transaction(|this| async {
      this.execute(buffer_cmd.as_str(), |_| Ok(())).await?;
      Ok(((), this))
    })
    .await?;
  buffer_cmd.clear();

  Ok(())
}

#[inline]
pub(crate) async fn _migrations_by_mg_version_query<E, D>(
  buffer_cmd: &mut String,
  executor: &mut E,
  mg_version: i32,
  results: &mut Vector<DbMigration>,
  schema_prefix: &str,
) -> crate::Result<()>
where
  D: Database<Aux = (), Error = crate::Error>,
  E: Executor<Database = D>,
  DbMigration: FromRecord<E::Database>,
{
  buffer_cmd.write_fmt(format_args!(
    "SELECT \
      _wtx_migration.version, \
      _wtx_migration_group.version as omg_version, \
      _wtx_migration_group.name as omg_name, \
      _wtx_migration.checksum, \
      _wtx_migration.created_on, \
      _wtx_migration.name, \
      _wtx_migration.repeatability \
    FROM \
      {schema_prefix}_wtx_migration_group \
    JOIN \
      {schema_prefix}_wtx_migration ON _wtx_migration._wtx_migration_omg_version = _wtx_migration_group.version \
    WHERE \
      _wtx_migration_group.version = {mg_version} \
    ORDER BY \
      _wtx_migration.version ASC;",
  ))?;
  executor
    .simple_entities(buffer_cmd, (), |result| {
      results.push(result)?;
      Ok(())
    })
    .await?;
  buffer_cmd.clear();
  Ok(())
}
