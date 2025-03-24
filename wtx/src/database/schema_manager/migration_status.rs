/// Status of a migration operation;
#[derive(Debug)]
pub struct MigrationStatus {
  /// The number of applied migrations performed in the current operation.
  pub curr_applied_migrations: u64,
  /// The number of migrations stored in the database after applying a operation.
  pub curr_db_migrations: u64,
  /// The migration group version
  pub mg_version: u32,
  /// The last migration version stored in the database after applying a operation.
  pub curr_last_db_migration_version: Option<u32>,
  /// The last migration version stored in the database before applying a operation.
  pub prev_last_db_migration_version: Option<u32>,
  /// The number of migrations stored in the database before applying a operation.
  pub prev_db_migrations: u64,
}
