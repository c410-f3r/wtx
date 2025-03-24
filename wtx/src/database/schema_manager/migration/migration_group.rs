use crate::{database::schema_manager::VersionTy, misc::Lease};

/// A set of unique migrations
///
/// * Types
///
/// S: Sequence of characters
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MigrationGroup<S> {
  name: S,
  version: VersionTy,
}

impl<S> MigrationGroup<S>
where
  S: Lease<str>,
{
  /// Creates a new instance from all necessary parameters.
  #[inline]
  pub const fn new(name: S, version: VersionTy) -> Self {
    Self { name, version }
  }

  /// Name
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::migration_group;
  /// assert_eq!(migration_group().name(), "initial");
  /// ```
  #[inline]
  pub fn name(&self) -> &str {
    self.name.lease()
  }

  /// Version
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::migration_group;
  /// assert_eq!(migration_group().version(), 1);
  /// ```
  #[inline]
  pub fn version(&self) -> VersionTy {
    self.version
  }
}
