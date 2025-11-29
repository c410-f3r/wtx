use crate::{
  collection::Vector,
  database::{
    DatabaseTy,
    schema_manager::{
      Commands, DbMigration, MigrationStatus, SchemaManagement, SchemaManagerError, UserMigration,
      UserMigrationGroup, VERSION,
    },
  },
  de::DEController,
  misc::{Lease, Usize},
};
use alloc::string::String;
#[cfg(feature = "std")]
use {
  crate::database::schema_manager::misc::{group_and_migrations_from_path, parse_root_toml},
  std::path::{Path, PathBuf},
};

type MigrationFromGroups<'slice, 'migration_group, 'migration_slice, DBS, S> =
  &'slice [(&'migration_group UserMigrationGroup<S>, &'migration_slice [UserMigration<DBS, S>])];

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Migrates everything inside a group that is greater than the last migration ID within the
  /// database
  #[inline]
  pub async fn migrate<'migration, DBS, I, S>(
    &mut self,
    mg: &UserMigrationGroup<S>,
    user_migrations: I,
  ) -> Result<MigrationStatus, <E::Database as DEController>::Error>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: IntoIterator<Item = &'migration UserMigration<DBS, S>>,
    I::IntoIter: Clone,
    S: Lease<str> + 'migration,
  {
    let mut buffer_cmd = String::new();
    let mut buffer_db_migrations = Vector::new();
    self.executor.create_wtx_tables().await?;
    self.executor.migrations(&mut buffer_cmd, mg, &mut buffer_db_migrations).await?;
    self
      .do_migrate((&mut buffer_cmd, &mut buffer_db_migrations), mg, user_migrations.into_iter())
      .await
  }

  /// Applies `migrate` to a set of migrations according to a given directory
  #[cfg(feature = "std")]
  #[inline]
  pub async fn migrate_from_dir(
    &mut self,
    path: &Path,
  ) -> Result<(), <E::Database as DEController>::Error> {
    self
      .do_migrate_from_dir((&mut String::new(), &mut Vector::new(), &mut Vector::new()), path)
      .await
  }

  /// Applies `migrate` to a set of migration groups according to the configuration file.
  #[cfg(feature = "std")]
  #[inline]
  pub async fn migrate_from_toml_path(
    &mut self,
    path: &Path,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let (mut migration_groups, _) = parse_root_toml(path)?;
    migration_groups.sort_unstable();
    self.migrate_from_groups_paths(&migration_groups).await?;
    Ok(())
  }

  /// Applies `migrate` to a set of migrations according to a given set of groups
  #[inline]
  pub async fn migrate_from_groups<DBS, S>(
    &mut self,
    groups: MigrationFromGroups<'_, '_, '_, DBS, S>,
  ) -> Result<(), <E::Database as DEController>::Error>
  where
    DBS: Lease<[DatabaseTy]>,
    S: Lease<str>,
  {
    let mut buffer_cmd = String::new();
    let mut buffer_db_migrations = Vector::new();
    let mut buffer_status = Vector::new();
    self.executor.create_wtx_tables().await?;
    buffer_status.clear();
    for (mg, m) in groups {
      self.executor.migrations(&mut buffer_cmd, mg, &mut buffer_db_migrations).await?;
      buffer_status
        .push(self.do_migrate((&mut buffer_cmd, &mut buffer_db_migrations), mg, m.iter()).await?)?;
    }
    Ok(())
  }

  /// Applies `migrate` to the set of provided migration groups paths.
  #[cfg(feature = "std")]
  #[inline]
  pub async fn migrate_from_groups_paths(
    &mut self,
    migration_groups: &[PathBuf],
  ) -> Result<(), <E::Database as DEController>::Error> {
    self.executor.create_wtx_tables().await?;
    crate::database::schema_manager::misc::is_sorted_and_unique(migration_groups)?;
    for mg in migration_groups {
      self
        .do_migrate_from_dir((&mut String::new(), &mut Vector::new(), &mut Vector::new()), mg)
        .await?;
    }
    Ok(())
  }

  async fn do_migrate<'migration, DBS, I, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    mg: &UserMigrationGroup<S>,
    user_migrations: I,
  ) -> Result<MigrationStatus, <E::Database as DEController>::Error>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    let filtered_by_db = Self::filter_by_db(user_migrations);
    Self::do_validate(buffer_db_migrations, filtered_by_db.clone())?;
    let curr_applied_migrations;
    let curr_last_db_migration_uid = filtered_by_db.clone().last().map(|el| el.uid());
    let prev_db_migrations = Usize::from(buffer_db_migrations.len()).into_u64();
    let mut prev_last_db_migration_uid = None;
    if let Some(last_db_mig) = buffer_db_migrations.last() {
      if last_db_mig.group().version() != VERSION {
        return Err(
          crate::Error::from(SchemaManagerError::DivergentGroupVersions(
            last_db_mig.group().version(),
            VERSION,
          ))
          .into(),
        );
      }
      let to_apply = filtered_by_db.filter(move |e| e.uid() > last_db_mig.uid());
      curr_applied_migrations = Usize::from(to_apply.clone().count()).into();
      prev_last_db_migration_uid = Some(last_db_mig.uid());
      self.executor.insert_migrations(buffer_cmd, mg, to_apply).await?;
    } else {
      curr_applied_migrations = Usize::from(filtered_by_db.clone().count()).into();
      self.executor.insert_migrations(buffer_cmd, mg, filtered_by_db).await?;
    }
    buffer_db_migrations.clear();
    Ok(MigrationStatus {
      curr_applied_migrations,
      curr_last_db_migration_uid,
      mg_uid: mg.uid(),
      prev_last_db_migration_uid,
      prev_db_migrations,
    })
  }

  #[cfg(feature = "std")]
  async fn do_migrate_from_dir(
    &mut self,
    (buffer_cmd, buffer_db_migrations, buffer_status): (
      &mut String,
      &mut Vector<DbMigration>,
      &mut Vector<MigrationStatus>,
    ),
    path: &Path,
  ) -> Result<(), <E::Database as DEController>::Error> {
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
