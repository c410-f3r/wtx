//! Schema Manager

#[macro_use]
mod macros;

mod commands;
pub mod doc_tests;
pub(crate) mod fixed_sql_commands;
#[cfg(all(feature = "_integration-tests", feature = "schema-manager-dev", test))]
mod integration_tests;
mod migration;
#[cfg(feature = "std")]
pub mod migration_parser;
pub mod misc;
mod repeatability;
mod schema_manager_error;
#[cfg(feature = "std")]
pub mod toml_parser;

use crate::{
  database::{executor::Executor, DatabaseTy, Identifier},
  misc::{Lease, Vector},
};
use alloc::string::String;
pub use commands::*;
use core::future::Future;
pub use migration::*;
pub use repeatability::Repeatability;
pub use schema_manager_error::SchemaManagerError;

/// Default batch size
pub const DEFAULT_BATCH_SIZE: usize = 128;
/// Default configuration file name.
pub const DEFAULT_CFG_FILE_NAME: &str = "wtx.toml";
pub(crate) const _WTX: &str = "wtx";
pub(crate) const _WTX_SCHEMA_PREFIX: &str = "_wtx.";

/// Useful in constant environments where the type must be explicitly declared.
///
/// ```ignore,rust
/// const MIGRATIONS: EmbeddedMigrationsTy = embed_migrations!("SOME_CFG_FILE.toml");
/// ```
pub type EmbeddedMigrationsTy = &'static [(
  &'static MigrationGroup<&'static str>,
  &'static [UserMigrationRef<'static, 'static>],
)];

/// Contains methods responsible to manage database migrations.
pub trait SchemaManagement: Executor {
  /// Clears all database resources.
  fn clear(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> impl Future<Output = crate::Result<()>>;

  /// Initial tables meant for initialization.
  fn create_wtx_tables(&mut self) -> impl Future<Output = crate::Result<()>>;

  /// Removes every migration of a given group `mg`` that is greater than `version`.
  fn delete_migrations<S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &MigrationGroup<S>,
    version: i32,
  ) -> impl Future<Output = crate::Result<()>>
  where
    S: Lease<str>;

  /// Inserts a new set of migrations,
  fn insert_migrations<'migration, DBS, I, S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &MigrationGroup<S>,
    migrations: I,
  ) -> impl Future<Output = crate::Result<()>>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration;

  /// Retrieves all migrations of the given `mg` group.
  fn migrations<S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &MigrationGroup<S>,
    results: &mut Vector<DbMigration>,
  ) -> impl Future<Output = crate::Result<()>>
  where
    S: Lease<str>;

  /// Retrieves all tables contained in a schema. If the implementation does not supports schemas,
  /// the parameter is ignored.
  fn table_names(
    &mut self,
    buffer_cmd: &mut String,
    results: &mut Vector<Identifier>,
    schema: &str,
  ) -> impl Future<Output = crate::Result<()>>;
}

impl SchemaManagement for () {
  #[inline]
  async fn clear(&mut self, _: (&mut String, &mut Vector<Identifier>)) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn create_wtx_tables(&mut self) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn delete_migrations<S>(
    &mut self,
    _: &mut String,
    _: &MigrationGroup<S>,
    _: i32,
  ) -> crate::Result<()>
  where
    S: Lease<str>,
  {
    Ok(())
  }

  #[inline]
  async fn insert_migrations<'migration, DBS, I, S>(
    &mut self,
    _: &mut String,
    _: &MigrationGroup<S>,
    _: I,
  ) -> crate::Result<()>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    Ok(())
  }

  #[inline]
  async fn migrations<S>(
    &mut self,
    _: &mut String,
    _: &MigrationGroup<S>,
    _: &mut Vector<DbMigration>,
  ) -> crate::Result<()>
  where
    S: Lease<str>,
  {
    Ok(())
  }

  #[inline]
  async fn table_names(
    &mut self,
    _: &mut String,
    _: &mut Vector<Identifier>,
    _: &str,
  ) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "postgres")]
mod postgres {
  use alloc::string::String;

  use crate::{
    database::{
      client::postgres::{Executor, ExecutorBuffer},
      schema_manager::{
        fixed_sql_commands::{
          _delete_migrations, _insert_migrations, _migrations_by_mg_version_query,
          postgres::{_clear, _table_names, _CREATE_MIGRATION_TABLES},
        },
        DbMigration, MigrationGroup, SchemaManagement, UserMigration, _WTX_SCHEMA_PREFIX,
      },
      DatabaseTy, Executor as _, Identifier,
    },
    misc::{Lease, LeaseMut, Stream, Vector},
  };

  impl<EB, STREAM> SchemaManagement for Executor<crate::Error, EB, STREAM>
  where
    EB: LeaseMut<ExecutorBuffer>,
    STREAM: Stream,
  {
    #[inline]
    async fn clear(&mut self, buffer: (&mut String, &mut Vector<Identifier>)) -> crate::Result<()> {
      _clear(buffer, self).await
    }

    #[inline]
    async fn create_wtx_tables(&mut self) -> crate::Result<()> {
      self.execute(_CREATE_MIGRATION_TABLES, |_| {}).await?;
      Ok(())
    }

    #[inline]
    async fn delete_migrations<S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &MigrationGroup<S>,
      version: i32,
    ) -> crate::Result<()>
    where
      S: Lease<str>,
    {
      _delete_migrations(buffer_cmd, self, mg, _WTX_SCHEMA_PREFIX, version).await
    }

    #[inline]
    async fn insert_migrations<'migration, DBS, I, S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &MigrationGroup<S>,
      migrations: I,
    ) -> crate::Result<()>
    where
      DBS: Lease<[DatabaseTy]> + 'migration,
      I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
      S: Lease<str> + 'migration,
    {
      _insert_migrations(buffer_cmd, self, mg, migrations, _WTX_SCHEMA_PREFIX).await
    }

    #[inline]
    async fn migrations<S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &MigrationGroup<S>,
      results: &mut Vector<DbMigration>,
    ) -> crate::Result<()>
    where
      S: Lease<str>,
    {
      _migrations_by_mg_version_query(buffer_cmd, self, mg.version(), results, _WTX_SCHEMA_PREFIX)
        .await
    }

    #[inline]
    async fn table_names(
      &mut self,
      buffer_cmd: &mut String,
      results: &mut Vector<Identifier>,
      schema: &str,
    ) -> crate::Result<()> {
      _table_names(buffer_cmd, self, results, schema).await
    }
  }
}
