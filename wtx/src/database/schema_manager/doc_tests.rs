#![expect(clippy::missing_panics_doc, reason = "examples never panic")]
#![expect(clippy::unwrap_used, reason = "intended for tests only")]

//! Instances mostly used for documentation tests

use crate::database::schema_manager::{MigrationGroup, UserMigrationRef};

/// ```rust
/// let _ = wtx::database::schema_manager::UserMigrationRef::from_user_parts(
///   &[],
///   "create_author",
///   None,
///   [
///     "CREATE TABLE author (id INT NOT NULL PRIMARY KEY, name VARCHAR(50) NOT NULL)",
///     "DROP TABLE author",
///   ],
///   1
/// );
/// ```
#[inline]
pub fn migration() -> UserMigrationRef<'static, 'static> {
  UserMigrationRef::from_user_parts(
    &[],
    "create_author",
    None,
    [
      "CREATE TABLE author (id INT NOT NULL PRIMARY KEY, name VARCHAR(50) NOT NULL)",
      "DROP TABLE author",
    ],
    1,
  )
  .unwrap()
}

/// ```rust
/// let _ = wtx::database::schema_manager::MigrationGroup::new("initial", 1);
/// ```
#[inline]
pub fn migration_group() -> MigrationGroup<&'static str> {
  MigrationGroup::new("initial", 1)
}
