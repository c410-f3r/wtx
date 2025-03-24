use crate::{
  database::{
    DatabaseTy,
    schema_manager::{
      Commands, DbMigration, MigrationGroup, SchemaManagement, UserMigration, VersionTy,
    },
  },
  misc::{DEController, Lease, Vector},
};
use alloc::string::String;
#[cfg(feature = "std")]
use {
  crate::database::schema_manager::SchemaManagerError,
  crate::database::schema_manager::misc::{group_and_migrations_from_path, parse_root_toml},
  std::path::Path,
};

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Rollbacks the migrations of a group to a given `version`.
  ///
  /// Before issuing a rollback, all migrations are validated.
  #[inline]
  pub async fn rollback<'migration, DBS, I, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    mg: &MigrationGroup<S>,
    migrations: I,
    version: VersionTy,
  ) -> Result<(), <E::Database as DEController>::Error>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    self.executor.migrations(buffer_cmd, mg, buffer_db_migrations).await?;
    let filtered_by_db = Self::filter_by_db(migrations);
    Self::do_validate(buffer_db_migrations, filtered_by_db.clone())?;
    for elem in filtered_by_db.map(UserMigration::sql_down) {
      buffer_cmd.push_str(elem);
    }
    self
      .executor
      .transaction(|this| async {
        this.execute(buffer_cmd.as_str(), |_| Ok(())).await?;
        Ok(((), this))
      })
      .await?;
    buffer_cmd.clear();
    self.executor.delete_migrations(buffer_cmd, mg, version).await?;
    buffer_db_migrations.clear();
    Ok(())
  }

  /// Applies `rollback` to a set of groups according to the configuration file
  #[inline]
  #[cfg(feature = "std")]
  pub async fn rollback_from_toml(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    path: &Path,
    versions: &[VersionTy],
  ) -> Result<(), <E::Database as DEController>::Error> {
    let (mut migration_groups, _) = parse_root_toml(path)?;
    if migration_groups.len() != versions.len() {
      return Err(crate::Error::from(SchemaManagerError::DifferentRollbackVersions).into());
    }
    migration_groups.sort_by(|a, b| b.cmp(a));
    for (mg, &version) in migration_groups.into_iter().zip(versions) {
      self.do_rollback_from_dir((buffer_cmd, buffer_db_migrations), &mg, version).await?;
    }
    Ok(())
  }

  /// Applies `rollback` to a set of migrations according to a given directory
  #[inline]
  #[cfg(feature = "std")]
  pub async fn rollback_from_dir(
    &mut self,
    buffer: (&mut String, &mut Vector<DbMigration>),
    path: &Path,
    version: VersionTy,
  ) -> Result<(), <E::Database as DEController>::Error> {
    self.do_rollback_from_dir(buffer, path, version).await
  }

  #[inline]
  #[cfg(feature = "std")]
  async fn do_rollback_from_dir(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    path: &Path,
    version: VersionTy,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let opt = group_and_migrations_from_path(path, |a, b| b.cmp(a));
    let Ok((mg, mut migrations)) = opt else { return Ok(()) };
    let mut tmp_migrations = Vector::new();
    loop_files!(
      tmp_migrations,
      migrations,
      self.batch_size(),
      self
        .rollback((buffer_cmd, buffer_db_migrations), &mg, tmp_migrations.iter(), version)
        .await?
    );
    Ok(())
  }
}
