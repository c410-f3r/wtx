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
mod migration_status;
pub mod misc;
mod repeatability;
mod schema_manager_error;
#[cfg(feature = "std")]
pub mod toml_parser;

use crate::{
  collection::Vector,
  database::{DatabaseTy, Identifier, executor::Executor},
  de::DEController,
  misc::Lease,
};
use alloc::string::String;
pub use commands::*;
pub use migration::*;
pub use migration_status::MigrationStatus;
pub use repeatability::Repeatability;
pub use schema_manager_error::SchemaManagerError;

/// Default batch size
pub const DEFAULT_BATCH_SIZE: usize = 128;
/// Default configuration file name.
pub const DEFAULT_CFG_FILE_NAME: &str = "wtx.toml";
/// Schema API version
pub const VERSION: u32 = 1;
pub(crate) const _WTX: &str = "wtx";
pub(crate) const _WTX_PREFIX: &str = "_wtx";
pub(crate) const _WTX_SCHEMA: &str = "_wtx.";

/// Useful in constant environments where the type must be explicitly declared.
///
/// ```ignore,rust
/// const MIGRATIONS: EmbeddedMigrationsTy = embed_migrations!("SOME_CFG_FILE.toml");
/// ```
pub type EmbeddedMigrationsTy = &'static [(
  &'static UserMigrationGroup<&'static str>,
  &'static [UserMigrationRef<'static, 'static>],
)];
/// Identifiers provided by users for migrations.
pub type Uid = u32;

/// Contains methods responsible to manage database migrations.
pub trait SchemaManagement: Executor {
  /// Retrieves all inserted elements.
  fn all_elements(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>;

  /// Cleans all database resources.
  fn clear(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>;

  /// Initial tables meant for initialization.
  fn create_wtx_tables(
    &mut self,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>;

  /// Removes every migration of a given group `mg`` that is greater than `uid`.
  fn delete_migrations<S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &UserMigrationGroup<S>,
    uid: Uid,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>
  where
    S: Lease<str>;

  /// Inserts a new set of migrations,
  fn insert_migrations<'migration, DBS, I, S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &UserMigrationGroup<S>,
    migrations: I,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration;

  /// Retrieves all migrations of the given `mg` group in ascending order.
  fn migrations<S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &UserMigrationGroup<S>,
    results: &mut Vector<DbMigration>,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>
  where
    S: Lease<str>;

  /// Retrieves all tables contained in a schema. If the implementation does not supports schemas,
  /// the parameter is ignored.
  fn table_names(
    &mut self,
    buffer_cmd: &mut String,
    results: &mut Vector<Identifier>,
    schema: &str,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>;
}

impl<T> SchemaManagement for &mut T
where
  T: SchemaManagement,
{
  #[inline]
  async fn all_elements(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> Result<(), <Self::Database as DEController>::Error> {
    (**self).all_elements(buffer).await
  }

  #[inline]
  async fn clear(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> Result<(), <Self::Database as DEController>::Error> {
    (**self).clear(buffer).await
  }

  #[inline]
  async fn create_wtx_tables(&mut self) -> Result<(), <Self::Database as DEController>::Error> {
    (**self).create_wtx_tables().await
  }

  #[inline]
  async fn delete_migrations<S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &UserMigrationGroup<S>,
    uid: Uid,
  ) -> Result<(), <Self::Database as DEController>::Error>
  where
    S: Lease<str>,
  {
    (**self).delete_migrations(buffer_cmd, mg, uid).await
  }

  #[inline]
  async fn insert_migrations<'migration, DBS, I, S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &UserMigrationGroup<S>,
    migrations: I,
  ) -> Result<(), <Self::Database as DEController>::Error>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    (**self).insert_migrations(buffer_cmd, mg, migrations).await
  }

  #[inline]
  async fn migrations<S>(
    &mut self,
    buffer_cmd: &mut String,
    mg: &UserMigrationGroup<S>,
    results: &mut Vector<DbMigration>,
  ) -> Result<(), <Self::Database as DEController>::Error>
  where
    S: Lease<str>,
  {
    (**self).migrations(buffer_cmd, mg, results).await
  }

  #[inline]
  async fn table_names(
    &mut self,
    buffer_cmd: &mut String,
    results: &mut Vector<Identifier>,
    schema: &str,
  ) -> Result<(), <Self::Database as DEController>::Error> {
    (**self).table_names(buffer_cmd, results, schema).await
  }
}

impl SchemaManagement for () {
  #[inline]
  async fn all_elements(&mut self, _: (&mut String, &mut Vector<Identifier>)) -> crate::Result<()> {
    Ok(())
  }

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
    _: &UserMigrationGroup<S>,
    _: Uid,
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
    _: &UserMigrationGroup<S>,
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
    _: &UserMigrationGroup<S>,
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

#[cfg(feature = "mysql")]
mod mysql {
  use crate::{
    collection::Vector,
    database::{
      DatabaseTy, Executor as _, Identifier,
      client::mysql::{ExecutorBuffer, MysqlExecutor},
      schema_manager::{
        DbMigration, SchemaManagement, Uid, UserMigration, UserMigrationGroup,
        fixed_sql_commands::{
          common::{delete_migrations, insert_migrations, migrations_by_mg_uid_query},
          mysql::{CREATE_MIGRATION_TABLES, clear, table_names},
        },
      },
    },
    misc::{Lease, LeaseMut},
    stream::Stream,
  };
  use alloc::string::String;

  impl<E, EB, STREAM> SchemaManagement for MysqlExecutor<E, EB, STREAM>
  where
    E: From<crate::Error>,
    EB: LeaseMut<ExecutorBuffer>,
    STREAM: Stream,
  {
    #[inline]
    async fn all_elements(
      &mut self,
      buffer: (&mut String, &mut Vector<Identifier>),
    ) -> Result<(), E> {
      self.table_names(buffer.0, buffer.1, "").await?;
      Ok(())
    }

    #[inline]
    async fn clear(&mut self, _: (&mut String, &mut Vector<Identifier>)) -> Result<(), E> {
      clear(self).await
    }

    #[inline]
    async fn create_wtx_tables(&mut self) -> Result<(), E> {
      self.execute_ignored(CREATE_MIGRATION_TABLES).await?;
      Ok(())
    }

    #[inline]
    async fn delete_migrations<S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &UserMigrationGroup<S>,
      uid: Uid,
    ) -> Result<(), E>
    where
      S: Lease<str>,
    {
      delete_migrations(buffer_cmd, self, mg, "", uid).await
    }

    #[inline]
    async fn insert_migrations<'migration, DBS, I, S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &UserMigrationGroup<S>,
      migrations: I,
    ) -> Result<(), E>
    where
      DBS: Lease<[DatabaseTy]> + 'migration,
      I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
      S: Lease<str> + 'migration,
    {
      insert_migrations(buffer_cmd, self, mg, migrations, "").await
    }

    #[inline]
    async fn migrations<S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &UserMigrationGroup<S>,
      results: &mut Vector<DbMigration>,
    ) -> Result<(), E>
    where
      S: Lease<str>,
    {
      migrations_by_mg_uid_query(buffer_cmd, self, mg.uid(), results, "").await
    }

    #[inline]
    async fn table_names(
      &mut self,
      _: &mut String,
      results: &mut Vector<Identifier>,
      _: &str,
    ) -> Result<(), E> {
      table_names(self, results).await
    }
  }
}

#[cfg(feature = "postgres")]
mod postgres {
  use crate::{
    collection::Vector,
    database::{
      DatabaseTy, Executor as _, Identifier,
      client::postgres::{ExecutorBuffer, PostgresExecutor},
      schema_manager::{
        _WTX_SCHEMA, DbMigration, SchemaManagement, Uid, UserMigration, UserMigrationGroup,
        fixed_sql_commands::{
          common::{delete_migrations, insert_migrations, migrations_by_mg_uid_query},
          postgres::{CREATE_MIGRATION_TABLES, all_elements, clear, table_names},
        },
      },
    },
    misc::{Lease, LeaseMut},
    stream::Stream,
  };
  use alloc::string::String;

  impl<E, EB, STREAM> SchemaManagement for PostgresExecutor<E, EB, STREAM>
  where
    E: From<crate::Error>,
    EB: LeaseMut<ExecutorBuffer>,
    STREAM: Stream,
  {
    #[inline]
    async fn all_elements(
      &mut self,
      (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
    ) -> Result<(), E> {
      all_elements(
        (buffer_cmd, buffer_idents),
        self,
        |_| Ok(()),
        |_| Ok(()),
        |_| Ok(()),
        |_| Ok(()),
        |_| Ok(()),
        |_| Ok(()),
        |_| Ok(()),
        |_| Ok(()),
      )
      .await?;
      Ok(())
    }

    #[inline]
    async fn clear(&mut self, buffer: (&mut String, &mut Vector<Identifier>)) -> Result<(), E> {
      clear(buffer, self).await
    }

    #[inline]
    async fn create_wtx_tables(&mut self) -> Result<(), E> {
      self.execute_ignored(CREATE_MIGRATION_TABLES).await?;
      Ok(())
    }

    #[inline]
    async fn delete_migrations<S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &UserMigrationGroup<S>,
      uid: Uid,
    ) -> Result<(), E>
    where
      S: Lease<str>,
    {
      delete_migrations(buffer_cmd, self, mg, _WTX_SCHEMA, uid).await
    }

    #[inline]
    async fn insert_migrations<'migration, DBS, I, S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &UserMigrationGroup<S>,
      migrations: I,
    ) -> Result<(), E>
    where
      DBS: Lease<[DatabaseTy]> + 'migration,
      I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
      S: Lease<str> + 'migration,
    {
      insert_migrations(buffer_cmd, self, mg, migrations, _WTX_SCHEMA).await
    }

    #[inline]
    async fn migrations<S>(
      &mut self,
      buffer_cmd: &mut String,
      mg: &UserMigrationGroup<S>,
      results: &mut Vector<DbMigration>,
    ) -> Result<(), E>
    where
      S: Lease<str>,
    {
      migrations_by_mg_uid_query(buffer_cmd, self, mg.uid(), results, _WTX_SCHEMA).await
    }

    #[inline]
    async fn table_names(
      &mut self,
      buffer_cmd: &mut String,
      results: &mut Vector<Identifier>,
      schema: &str,
    ) -> Result<(), E> {
      table_names(buffer_cmd, self, results, schema).await
    }
  }
}
