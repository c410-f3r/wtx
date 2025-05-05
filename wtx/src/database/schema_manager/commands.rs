#[cfg(feature = "schema-manager-dev")]
mod clear;
mod migrate;
mod rollback;
#[cfg(feature = "schema-manager-dev")]
mod seed;
mod validate;

use crate::{
  collection::Vector,
  database::{
    Database, DatabaseTy, Identifier,
    executor::Executor,
    schema_manager::{DEFAULT_BATCH_SIZE, SchemaManagement, UserMigration},
  },
  misc::Lease,
};
use alloc::string::String;

/// SQL commands facade
#[derive(Debug)]
pub struct Commands<E> {
  batch_size: usize,
  executor: E,
}

impl<E> Commands<E>
where
  E: Executor,
{
  /// Creates a new instance from a given Backend and batch size.
  #[inline]
  pub fn new(batch_size: usize, executor: E) -> Self {
    Self { batch_size, executor }
  }

  /// Creates a new instance from a given Backend.
  ///
  /// Batch size will default to 128.
  #[inline]
  pub fn with_executor(database: E) -> Self {
    Self { batch_size: DEFAULT_BATCH_SIZE, executor: database }
  }

  /// Batch size
  #[inline]
  pub fn batch_size(&self) -> usize {
    self.batch_size
  }

  /// Allows the access of the internal executor
  #[cfg(test)]
  #[cfg(any(feature = "mysql", feature = "postgres"))]
  pub(crate) fn executor_mut(&mut self) -> &mut E {
    &mut self.executor
  }

  fn filter_by_db<'migration, DBS, I, S>(
    migrations: I,
  ) -> impl Clone + Iterator<Item = &'migration UserMigration<DBS, S>>
  where
    DBS: Lease<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: Lease<str> + 'migration,
  {
    migrations.filter(move |m| {
      if m.dbs().is_empty() { true } else { m.dbs().contains(&<E::Database as Database>::TY) }
    })
  }
}

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Retrieves all inserted elements.
  pub async fn all_elements(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> crate::Result<()> {
    self.executor.all_elements(buffer).await
  }
}
