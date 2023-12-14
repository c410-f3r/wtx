#[cfg(feature = "sm-dev")]
mod clear;
mod migrate;
mod rollback;
#[cfg(feature = "sm-dev")]
mod seed;
mod validate;

use crate::database::{
  executor::Executor,
  sm::{UserMigration, DEFAULT_BATCH_SIZE},
  Database, DatabaseTy,
};

/// SQL commands facade
#[derive(Debug)]
pub struct Commands<E> {
  batch_size: usize,
  pub(crate) executor: E,
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

  #[inline]
  fn filter_by_db<'migration, DBS, I, S>(
    migrations: I,
  ) -> impl Clone + Iterator<Item = &'migration UserMigration<DBS, S>>
  where
    DBS: AsRef<[DatabaseTy]> + 'migration,
    I: Clone + Iterator<Item = &'migration UserMigration<DBS, S>>,
    S: AsRef<str> + 'migration,
  {
    migrations.filter(move |m| {
      if m.dbs().is_empty() {
        true
      } else {
        m.dbs().contains(&<E::Database as Database>::TY)
      }
    })
  }
}
