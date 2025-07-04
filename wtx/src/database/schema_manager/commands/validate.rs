use crate::{
  collection::{IndexedStorageMut, Vector},
  database::{
    DatabaseTy,
    schema_manager::{
      Commands, DbMigration, Repeatability, SchemaManagement, SchemaManagerError, UserMigration,
      UserMigrationGroup, misc::is_migration_divergent,
    },
  },
  misc::Lease,
};
use alloc::string::String;
#[cfg(feature = "std")]
use {
  crate::database::schema_manager::misc::{group_and_migrations_from_path, parse_root_toml},
  std::path::Path,
};

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Verifies if the provided migrations are a superset of the migrations within the database
  /// by verification their checksums.
  #[inline]
  pub async fn validate<'migration, DBS, I, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    mg: &UserMigrationGroup<S>,
    migrations: I,
  ) -> crate::Result<()>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    self.executor.migrations(buffer_cmd, mg, buffer_db_migrations).await?;
    Self::do_validate(buffer_db_migrations, Self::filter_by_db(migrations))?;
    buffer_db_migrations.clear();
    Ok(())
  }

  /// Applies `validate` to a set of groups according to the configuration file
  #[inline]
  #[cfg(feature = "std")]
  pub async fn validate_from_toml(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    path: &Path,
  ) -> crate::Result<()> {
    let (mut migration_groups, _) = parse_root_toml(path)?;
    migration_groups.sort_unstable();
    for mg in migration_groups.into_iter() {
      self.do_validate_from_dir((buffer_cmd, buffer_db_migrations), &mg).await?;
    }
    Ok(())
  }

  /// Applies `validate` to a set of migrations according to a given directory
  #[inline]
  #[cfg(feature = "std")]
  pub async fn validate_from_dir(
    &mut self,
    buffer: (&mut String, &mut Vector<DbMigration>),
    path: &Path,
  ) -> crate::Result<()> {
    self.do_validate_from_dir(buffer, path).await
  }

  pub(crate) fn do_validate<'migration, DBS, S, I>(
    db_migrations: &[DbMigration],
    migrations: I,
  ) -> crate::Result<()>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    let mut migrations_len: usize = 0;
    for migration in migrations {
      match migration.repeatability() {
        Some(Repeatability::Always) => {}
        _ => {
          if is_migration_divergent(db_migrations, migration) {
            return Err(SchemaManagerError::DivergentMigration(migration.uid()).into());
          }
        }
      }
      migrations_len = migrations_len.saturating_add(1);
    }
    if migrations_len < db_migrations.len() {
      return Err(
        SchemaManagerError::DivergentMigrationsNum {
          expected: db_migrations.len().try_into().unwrap_or(u32::MAX),
          received: migrations_len.try_into().unwrap_or(u32::MAX),
        }
        .into(),
      );
    }
    Ok(())
  }

  #[cfg(feature = "std")]
  async fn do_validate_from_dir(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    path: &Path,
  ) -> crate::Result<()> {
    let opt = group_and_migrations_from_path(path, Ord::cmp);
    let Ok((mg, mut migrations)) = opt else { return Ok(()) };
    self.executor.migrations(buffer_cmd, &mg, buffer_db_migrations).await?;
    let mut tmp_migrations = Vector::new();
    loop_files!(
      tmp_migrations,
      migrations,
      self.batch_size(),
      Self::do_validate(buffer_db_migrations, tmp_migrations.iter())?
    );
    buffer_db_migrations.clear();
    Ok(())
  }
}
