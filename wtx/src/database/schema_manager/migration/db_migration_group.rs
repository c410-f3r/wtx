use crate::{
  database::schema_manager::{Uid, migration::migration_group_common::MigrationGroupCommon},
  misc::Lease,
};

/// A group of migrations retrieved from a database.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DbMigrationGroup<S> {
  common: MigrationGroupCommon<S>,
  version: u32,
}

impl<S> DbMigrationGroup<S>
where
  S: Lease<str>,
{
  /// Creates a new instance from all necessary parameters.
  #[inline]
  pub const fn new(name: S, uid: Uid, version: u32) -> Self {
    Self { common: MigrationGroupCommon::new(name, uid), version }
  }

  /// Name
  #[inline]
  pub fn name(&self) -> &str {
    self.common.name.lease()
  }

  /// User ID
  #[inline]
  pub const fn uid(&self) -> Uid {
    self.common.uid
  }

  /// Used to track API changes.
  #[inline]
  pub const fn version(&self) -> u32 {
    self.version
  }
}
