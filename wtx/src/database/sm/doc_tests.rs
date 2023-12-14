#![allow(
  // Examples never panic
  clippy::missing_panics_doc,
  // Intended for tests only
  clippy::unwrap_used
)]

//! Instances mostly used for documentation tests

use crate::database::sm::{MigrationGroup, UserMigrationRef};

/// ```rust
/// let _ = wtx::database::sm::UserMigrationRef::from_user_parts(
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
/// let _ = wtx::database::sm::MigrationGroup::new("initial", 1);
/// ```
#[inline]
pub fn migration_group() -> MigrationGroup<&'static str> {
  MigrationGroup::new("initial", 1)
}
