use crate::{
  collection::Vector,
  database::{
    DatabaseTy,
    schema_manager::{
      Commands, DbMigration, SchemaManagement, Uid, UserMigration, UserMigrationGroup,
    },
  },
  de::DEController,
  misc::Lease,
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
  /// Rollbacks all migrations of a group after the specific user ID that identifies a
  /// migration group.
  ///
  /// Before issuing a rollback, all migrations are validated.
  #[inline]
  pub async fn rollback<'migration, DBS, I, S>(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    mg: &UserMigrationGroup<S>,
    migrations: I,
    uid: Uid,
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
    self.executor.delete_migrations(buffer_cmd, mg, uid).await?;
    buffer_db_migrations.clear();
    Ok(())
  }

  /// Applies `rollback` to a set of groups according to the configuration file.
  ///
  /// Each `uids` corresponds to each migration group in ascending order. If `None`, then
  /// all migrations will be reverted.
  #[inline]
  #[cfg(feature = "std")]
  pub async fn rollback_from_toml(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    path: &Path,
    uids: Option<&[Uid]>,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let (mut migration_groups, _) = parse_root_toml(path)?;
    migration_groups.sort_by(|a, b| b.cmp(a));
    if let Some(elem) = uids {
      if migration_groups.len() != elem.len() {
        return Err(crate::Error::from(SchemaManagerError::DifferentRollbackUids).into());
      }
      for (mg, &uid) in migration_groups.into_iter().zip(elem) {
        self.rollback_from_dir((buffer_cmd, buffer_db_migrations), &mg, uid).await?;
      }
    } else {
      let iter = (0..migration_groups.len()).map(|_| 0);
      for (mg, uid) in migration_groups.into_iter().zip(iter) {
        self.rollback_from_dir((buffer_cmd, buffer_db_migrations), &mg, uid).await?;
      }
    };
    Ok(())
  }

  /// Applies `rollback` to a set of migrations according to a given directory
  #[inline]
  #[cfg(feature = "std")]
  pub async fn rollback_from_dir(
    &mut self,
    (buffer_cmd, buffer_db_migrations): (&mut String, &mut Vector<DbMigration>),
    path: &Path,
    uid: Uid,
  ) -> Result<(), <E::Database as DEController>::Error> {
    let Ok((mg, mut migrations)) = group_and_migrations_from_path(path, |a, b| b.cmp(a)) else {
      return Ok(());
    };
    let mut tmp_migrations = Vector::new();
    loop_files!(
      tmp_migrations,
      migrations,
      self.batch_size(),
      self.rollback((buffer_cmd, buffer_db_migrations), &mg, tmp_migrations.iter(), uid).await?
    );
    Ok(())
  }
}
