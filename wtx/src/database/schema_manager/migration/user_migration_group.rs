use crate::{
  database::schema_manager::{Uid, migration::migration_group_common::MigrationGroupCommon},
  misc::Lease,
};

/// A group of migrations.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UserMigrationGroup<S> {
  common: MigrationGroupCommon<S>,
}

impl<S> UserMigrationGroup<S>
where
  S: Lease<str>,
{
  /// Creates a new instance from all necessary parameters.
  #[inline]
  pub const fn new(name: S, uid: Uid) -> Self {
    Self { common: MigrationGroupCommon::new(name, uid) }
  }

  /// Name
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration_group;
  /// assert_eq!(user_migration_group().name(), "initial");
  /// ```
  #[inline]
  pub fn name(&self) -> &str {
    self.common.name.lease()
  }

  /// User ID
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::user_migration_group;
  /// assert_eq!(user_migration_group().uid(), 1);
  /// ```
  #[inline]
  pub fn uid(&self) -> Uid {
    self.common.uid
  }
}
