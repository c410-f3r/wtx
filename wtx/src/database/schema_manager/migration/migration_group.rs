use crate::{database::schema_manager::Uid, misc::Lease};

/// A set of unique migrations
///
/// * Types
///
/// S: Sequence of characters
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MigrationGroup<S> {
  name: S,
  uid: Uid,
}

impl<S> MigrationGroup<S>
where
  S: Lease<str>,
{
  /// Creates a new instance from all necessary parameters.
  #[inline]
  pub const fn new(name: S, uid: Uid) -> Self {
    Self { name, uid }
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

  /// Used ID
  ///
  /// # Example
  ///
  /// ```rust
  /// use wtx::database::schema_manager::doc_tests::migration_group;
  /// assert_eq!(migration_group().uid(), 1);
  /// ```
  #[inline]
  pub fn uid(&self) -> Uid {
    self.uid
  }
}
