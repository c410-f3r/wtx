use crate::database::{
  sm::{
    migration::MigrationCommon,
    misc::{calc_checksum, is_sorted_and_unique},
    Repeatability,
  },
  DatabaseTy,
};
use alloc::string::String;
use arrayvec::ArrayVec;

/// UserMigration - Owned
pub type UserMigrationOwned = UserMigration<ArrayVec<DatabaseTy, { DatabaseTy::len() }>, String>;
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
  DBS: AsRef<[DatabaseTy]>,
  S: AsRef<str>,
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
    version: i32,
  ) -> Self {
    Self {
      dbs,
      common: MigrationCommon { checksum, name, repeatability, version },
      sql_down,
      sql_up,
    }
  }

  /// Creates a new instance from all necessary input parameters that a user should provide.
  #[inline]
  pub fn from_user_parts(
    dbs: DBS,
    name: S,
    repeatability: Option<Repeatability>,
    [sql_up, sql_down]: [S; 2],
    version: i32,
  ) -> crate::Result<Self> {
    is_sorted_and_unique(dbs.as_ref())?;
    let checksum = calc_checksum(name.as_ref(), sql_up.as_ref(), sql_down.as_ref(), version);
    Ok(Self {
      dbs,
      common: MigrationCommon { checksum, name, repeatability, version },
      sql_down,
      sql_up,
    })
  }

  /// Checksum
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// assert_eq!(migration().checksum(), 9297329847391907999)
  /// ```
  #[inline]
  pub fn checksum(&self) -> u64 {
    self.common.checksum
  }

  /// Databases
  ///
  /// An empty slice means all databases
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// assert_eq!(migration().dbs(), [])
  /// ```
  #[inline]
  pub fn dbs(&self) -> &[DatabaseTy] {
    self.dbs.as_ref()
  }

  /// Name
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// assert_eq!(migration().name(), "create_author")
  /// ```
  #[inline]
  pub fn name(&self) -> &str {
    self.common.name.as_ref()
  }

  /// If this is a repeatable migration, returns its type.
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// assert_eq!(migration().repeatability(), None)
  /// ```
  #[inline]
  pub fn repeatability(&self) -> Option<Repeatability> {
    self.common.repeatability
  }

  /// Raw SQL for rollbacks
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// assert_eq!(migration().sql_down(), "DROP TABLE author");
  /// ```
  #[inline]
  pub fn sql_down(&self) -> &str {
    self.sql_down.as_ref()
  }

  /// Raw SQL for migrations
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// let mg = assert_eq!(
  ///   migration().sql_up(),
  ///   "CREATE TABLE author (id INT NOT NULL PRIMARY KEY, name VARCHAR(50) NOT NULL)"
  /// );
  #[inline]
  pub fn sql_up(&self) -> &str {
    self.sql_up.as_ref()
  }

  /// UserMigration version
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::sm::doc_tests::migration;
  /// assert_eq!(migration().version(), 1)
  /// ```
  #[inline]
  pub fn version(&self) -> i32 {
    self.common.version
  }
}
