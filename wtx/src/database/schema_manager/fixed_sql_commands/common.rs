use crate::{
  collection::Vector,
  database::{
    Database, DatabaseTy, FromRecords,
    executor::Executor,
    schema_manager::{DbMigration, Uid, UserMigration, UserMigrationGroup, VERSION},
  },
  misc::{DEController, Lease},
};
use alloc::string::String;
use core::fmt::Write;

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[inline]
pub(crate) async fn delete_migrations<E, S>(
  buffer_cmd: &mut String,
  executor: &mut E,
  mg: &UserMigrationGroup<S>,
  schema_prefix: &str,
  uid: Uid,
) -> Result<(), <E::Database as DEController>::Error>
where
  E: Executor,
  S: Lease<str>,
{
  buffer_cmd.write_fmt(format_args!(
      "DELETE FROM {schema_prefix}_wtx_migration WHERE _wtx_migration_mg_uid = {mg_uid} AND uid > {uid}",
      mg_uid = mg.uid(),
    )).map_err(crate::Error::from)?;
  executor.execute(buffer_cmd.as_str(), |_| Ok(())).await?;
  buffer_cmd.clear();
  Ok(())
}

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[inline]
pub(crate) async fn insert_migrations<'migration, DBS, E, I, S>(
  buffer_cmd: &mut String,
  executor: &mut E,
  mg: &UserMigrationGroup<S>,
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
      "INSERT INTO {schema_prefix}_wtx_migration_group (uid, name, version)
      SELECT * FROM (SELECT {mg_uid} AS uid, '{mg_name}' AS name, {VERSION} AS version) AS tmp
      WHERE NOT EXISTS (
        SELECT 1 FROM {schema_prefix}_wtx_migration_group WHERE uid = {mg_uid}
      );",
      mg_name = mg.name(),
      mg_uid = mg.uid(),
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
          uid, _wtx_migration_mg_uid, checksum, name
        ) VALUES (
          {m_uid}, {mg_uid}, '{m_checksum}', '{m_name}'
        );",
        m_checksum = migration.checksum(),
        m_name = migration.name(),
        m_uid = migration.uid(),
        mg_uid = mg.uid(),
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

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[inline]
pub(crate) async fn migrations_by_mg_uid_query<'exec, E, D>(
  buffer_cmd: &mut String,
  executor: &'exec mut E,
  mg_uid: Uid,
  results: &mut Vector<DbMigration>,
  schema_prefix: &str,
) -> crate::Result<()>
where
  D: Database<Aux = (), Error = crate::Error>,
  E: Executor<Database = D>,
  DbMigration: FromRecords<'exec, E::Database>,
{
  buffer_cmd.write_fmt(format_args!(
      "SELECT \
        _wtx_migration.uid, \
        _wtx_migration_group.uid as mg_uid, \
        _wtx_migration_group.name as mg_name, \
        _wtx_migration_group.version as mg_version, \
        _wtx_migration.checksum, \
        _wtx_migration.created_on, \
        _wtx_migration.name, \
        _wtx_migration.repeatability \
      FROM \
        {schema_prefix}_wtx_migration_group \
      JOIN \
        {schema_prefix}_wtx_migration ON _wtx_migration._wtx_migration_mg_uid = _wtx_migration_group.uid \
      WHERE \
        _wtx_migration_group.uid = {mg_uid} \
      ORDER BY \
        _wtx_migration.uid ASC;",
    ))?;
  for elem in DbMigration::many(
    &executor.fetch_many_with_stmt(buffer_cmd.as_str(), (), |_elem| Ok(())).await?,
  ) {
    if let Err(elem) = results.push(elem?) {
      buffer_cmd.clear();
      return Err(elem);
    }
  }
  buffer_cmd.clear();
  Ok(())
}
