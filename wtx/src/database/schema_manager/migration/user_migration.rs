use crate::{
  collection::ArrayVectorU8,
  database::{
    DatabaseTy,
    schema_manager::{
      Repeatability, Uid,
      migration::migration_common::MigrationCommon,
      misc::{calc_checksum, is_sorted_and_unique},
    },
  },
  misc::Lease,
};
use alloc::string::String;

/// UserMigration - Owned
pub type UserMigrationOwned =
  UserMigration<ArrayVectorU8<DatabaseTy, { DatabaseTy::len() }>, String>;
/// UserMigration - Reference
pub type UserMigrationRef<'dbs, 'str> = UserMigration<&'dbs [DatabaseTy], &'str str>;

/// A migration that is intended to be inserted into a database.
///
/// * Types
///
/// DBS: Databases
/// S: String
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UserMigration<DBS, S> {
  common: MigrationCommon<S>,
  dbs: DBS,
  sql_down: S,
  sql_up: S,
}

impl<DBS, S> UserMigration<DBS, S>
where
  DBS: Lease<[DatabaseTy]>,
  S: Lease<str>,
{
  /// Creates a new instance from all necessary parameters, including internal ones.
  #[inline]
  pub const fn from_all_parts(
    checksum: u64,
    dbs: DBS,
    name: S,
    repeatability: Option<Repeatability>,
    sql_down: S,
    sql_up: S,
    uid: Uid,
  ) -> Self {
    Self { dbs, common: MigrationCommon { checksum, name, repeatability, uid }, sql_down, sql_up }
  }

  /// Creates a new instance from all necessary input parameters that a user should provide.
  #[inline]
  pub fn from_user_parts(
    dbs: DBS,
    name: S,
    repeatability: Option<Repeatability>,
    [sql_up, sql_down]: [S; 2],
    uid: Uid,
  ) -> crate::Result<Self> {
    is_sorted_and_unique(dbs.lease())?;
    let checksum = calc_checksum(name.lease(), sql_up.lease(), sql_down.lease(), uid);
    Ok(Self {
      dbs,
      common: MigrationCommon { checksum, name, repeatability, uid },
      sql_down,
      sql_up,
    })
  }

  /// Checksum
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// assert_eq!(user_migration().checksum(), 9297329847391907999)
  /// ```
  #[inline]
  pub const fn checksum(&self) -> u64 {
    self.common.checksum
  }

  /// Databases
  ///
  /// An empty slice means all databases
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// assert_eq!(user_migration().dbs(), [])
  /// ```
  #[inline]
  pub fn dbs(&self) -> &[DatabaseTy] {
    self.dbs.lease()
  }

  /// Name
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// assert_eq!(user_migration().name(), "create_author")
  /// ```
  #[inline]
  pub fn name(&self) -> &str {
    self.common.name.lease()
  }

  /// If this is a repeatable migration, returns its type.
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// assert_eq!(user_migration().repeatability(), None)
  /// ```
  #[inline]
  pub const fn repeatability(&self) -> Option<Repeatability> {
    self.common.repeatability
  }

  /// Raw SQL for rollbacks
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// assert_eq!(user_migration().sql_down(), "DROP TABLE author");
  /// ```
  #[inline]
  pub fn sql_down(&self) -> &str {
    self.sql_down.lease()
  }

  /// Raw SQL for migrations
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// let mg = assert_eq!(
  ///   user_migration().sql_up(),
  ///   "CREATE TABLE author (id INT NOT NULL PRIMARY KEY, name VARCHAR(50) NOT NULL)"
  /// );
  #[inline]
  pub fn sql_up(&self) -> &str {
    self.sql_up.lease()
  }

  /// User ID
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration;
  /// assert_eq!(user_migration().uid(), 1)
  /// ```
  #[inline]
  pub const fn uid(&self) -> Uid {
    self.common.uid
  }
}
