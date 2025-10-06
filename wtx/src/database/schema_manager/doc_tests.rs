#![expect(clippy::missing_panics_doc, reason = "examples never panic")]
#![expect(clippy::unwrap_used, reason = "intended for tests only")]

//! Instances mostly used for documentation tests

use crate::database::schema_manager::{UserMigrationGroup, UserMigrationRef};

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
pub fn user_migration() -> UserMigrationRef<'static, 'static> {
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
/// let _ = wtx::database::schema_manager::UserMigrationGroup::new("initial", 1);
/// ```
#[inline]
pub const fn user_migration_group() -> UserMigrationGroup<&'static str> {
  UserMigrationGroup::new("initial", 1)
}
