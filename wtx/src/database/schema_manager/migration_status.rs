/// Status of a migration operation;
#[derive(Debug)]
pub struct MigrationStatus {
  /// The number of applied migrations performed in the current operation.
  pub curr_applied_migrations: u64,
  /// The last migration ID stored in the database after applying a operation.
  pub curr_last_db_migration_uid: Option<u32>,
  /// The migration group user ID
  pub mg_uid: u32,
  /// The last migration ID stored in the database before applying a operation.
  pub prev_last_db_migration_uid: Option<u32>,
  /// The number of migrations stored in the database before applying a operation.
  pub prev_db_migrations: u64,
}
