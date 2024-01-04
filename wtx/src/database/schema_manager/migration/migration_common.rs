use crate::database::schema_manager::Repeatability;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MigrationCommon<S> {
  pub(crate) checksum: u64,
  pub(crate) name: S,
  pub(crate) repeatability: Option<Repeatability>,
  pub(crate) version: i32,
}
