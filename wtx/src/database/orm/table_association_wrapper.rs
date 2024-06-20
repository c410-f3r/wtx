use crate::{
  database::orm::{Table, TableAssociation, TableParams},
  misc::LeaseMut,
};
use core::fmt::{Debug, Formatter};

/// A helper structure to manually implement [TableAssociations].
pub struct TableAssociationWrapper<'entity, T, TS>
where
  T: Table<'entity>,
  TS: LeaseMut<[TableParams<'entity, T>]>,
{
  /// See [TableAssociation]
  pub association: TableAssociation,
  /// Used to construct SELECT operations
  pub guide: TableParams<'entity, T>,
  /// A storage of zero, one or many tables used for INSERT and UPDATE operations
  pub tables: TS,
}

impl<'entity, T, TS> Debug for TableAssociationWrapper<'entity, T, TS>
where
  T: Debug + Table<'entity>,
  T::Associations: Debug,
  T::Fields: Debug,
  TS: Debug + LeaseMut<[TableParams<'entity, T>]>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("TableAssociationWrapper")
      .field("association", &self.association)
      .field("guide", &self.guide)
      .field("tables", &self.tables)
      .finish()
  }
}
