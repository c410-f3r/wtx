use crate::{
  database::{
    DatabaseTy,
    schema_manager::{
      Commands, DbMigration, MigrationGroup, MigrationStatus, SchemaManagement, UserMigration,
    },
  },
  misc::{Lease, Usize, Vector},
};
use alloc::string::String;
#[cfg(feature = "std")]
use {
  crate::database::schema_manager::misc::{group_and_migrations_from_path, parse_root_toml},
  std::path::{Path, PathBuf},
};

type MigrationFromGroups<'slice, 'migration_group, 'migration_slice, DBS, S> =
  &'slice [(&'migration_group MigrationGroup<S>, &'migration_slice [UserMigration<DBS, S>])];

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Migrates everything inside a group that is greater than the last migration version within the
  /// database
  #[inline]
  pub async fn migrate<'migration, DBS, I, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    mg: &MigrationGroup<S>,
    user_migrations: I,
  ) -> crate::Result<MigrationStatus>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    buffer_db_migrations.clear();
    self.executor.create_wtx_tables().await?;
    self.executor.migrations(buffer_cmd, mg, buffer_db_migrations).await?;
    self.do_migrate((buffer_cmd, buffer_db_migrations), mg, user_migrations).await
  }

  /// Applies `migrate` to a set of migrations according to a given directory
  #[cfg(feature = "std")]
  #[inline]
  pub async fn migrate_from_dir(
    &mut self,
    buffers: (&mut String, &mut Vector<DbMigration>, &mut Vector<MigrationStatus>),
    path: &Path,
  ) -> crate::Result<()> {
    self.executor.create_wtx_tables().await?;
    self.do_migrate_from_dir(buffers, path).await
  }

  /// Applies `migrate` to a set of migration groups according to the configuration file.
  #[cfg(feature = "std")]
  #[inline]
  pub async fn migrate_from_toml_path(
    &mut self,
    buffer: (&mut String, &mut Vector<DbMigration>, &mut Vector<MigrationStatus>),
    path: &Path,
  ) -> crate::Result<()> {
    let (mut migration_groups, _) = parse_root_toml(path)?;
    migration_groups.sort_unstable();
    self.migrate_from_groups_paths(buffer, &migration_groups).await?;
    Ok(())
  }

  /// Applies `migrate` to a set of migrations according to a given set of groups
  #[inline]
  pub async fn migrate_from_groups<DBS, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations, buffer_status): (
      &mut String,
      &mut Vector<DbMigration>,
      &mut Vector<MigrationStatus>,
    ),
    groups: MigrationFromGroups<'_, '_, '_, DBS, S>,
  ) -> crate::Result<()>
  where
    DBS: Lease<[DatabaseTy]>,
    S: Lease<str>,
  {
    self.executor.create_wtx_tables().await?;
    buffer_status.clear();
    for (mg, m) in groups {
      self.executor.migrations(buffer_cmd, mg, buffer_db_migrations).await?;
      buffer_status
        .push(self.do_migrate((buffer_cmd, buffer_db_migrations), mg, m.iter()).await?)?;
    }
    Ok(())
  }

  /// Applies `migrate` to the set of provided migration groups paths.
  #[cfg(feature = "std")]
  #[inline]
  pub async fn migrate_from_groups_paths(
    &mut self,
    (buffer_cmd, buffer_db_migrations, buffer_status): (
      &mut String,
      &mut Vector<DbMigration>,
      &mut Vector<MigrationStatus>,
    ),
    migration_groups: &[PathBuf],
  ) -> crate::Result<()> {
    self.executor.create_wtx_tables().await?;
    crate::database::schema_manager::misc::is_sorted_and_unique(migration_groups)?;
    for mg in migration_groups {
      self.do_migrate_from_dir((buffer_cmd, buffer_db_migrations, buffer_status), mg).await?;
    }
    Ok(())
  }

  #[inline]
  async fn do_migrate<'migration, DBS, I, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    mg: &MigrationGroup<S>,
    user_migrations: I,
  ) -> crate::Result<MigrationStatus>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    let filtered_by_db = Self::filter_by_db(user_migrations);
    Self::do_validate(buffer_db_migrations, filtered_by_db.clone())?;
    let curr_applied_migrations;
    let curr_last_db_migration_version = filtered_by_db.clone().last().map(|el| el.version());
    let prev_db_migrations = Usize::from(buffer_db_migrations.len()).into_u64();
    let mut prev_last_db_migration_version = None;
    if let Some(last_db_mig_version) = buffer_db_migrations.last().map(DbMigration::version) {
      let to_apply = filtered_by_db.filter(move |e| e.version() > last_db_mig_version);
      curr_applied_migrations = Usize::from(to_apply.clone().count()).into();
      prev_last_db_migration_version = Some(last_db_mig_version);
      self.executor.insert_migrations(buffer_cmd, mg, to_apply).await?;
    } else {
      curr_applied_migrations = Usize::from(filtered_by_db.clone().count()).into();
      self.executor.insert_migrations(buffer_cmd, mg, filtered_by_db).await?;
    }
    buffer_db_migrations.clear();
    Ok(MigrationStatus {
      curr_applied_migrations,
      curr_last_db_migration_version,
      mg_version: mg.version(),
      prev_last_db_migration_version,
      prev_db_migrations,
    })
  }

  #[cfg(feature = "std")]
  #[inline]
  async fn do_migrate_from_dir(
    &mut self,
    (buffer_cmd, buffer_db_migrations, buffer_status): (
      &mut String,
      &mut Vector<DbMigration>,
      &mut Vector<MigrationStatus>,
    ),
    path: &Path,
  ) -> crate::Result<()> {
    let (mg, mut migrations) = group_and_migrations_from_path(path, Ord::cmp)?;
    self.executor.migrations(buffer_cmd, &mg, buffer_db_migrations).await?;
    let mut tmp_migrations = Vector::new();
    buffer_status.clear();
    loop_files!(
      tmp_migrations,
      migrations,
      self.batch_size(),
      buffer_status.push(
        self.do_migrate((buffer_cmd, buffer_db_migrations), &mg, tmp_migrations.iter()).await?
      )?
    );
    Ok(())
  }
}
