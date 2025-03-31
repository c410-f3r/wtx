use crate::{database::schema_manager::Uid, misc::Lease};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MigrationGroupCommon<S> {
  pub(crate) name: S,
  pub(crate) uid: Uid,
}

impl<S> MigrationGroupCommon<S>
where
  S: Lease<str>,
{
  #[inline]
  pub(crate) const fn new(name: S, uid: Uid) -> Self {
    Self { name, uid }
  }
}
