use crate::{
  database::orm::{Table, TableAssociation, TableParams},
  misc::{Lease, SingleTypeStorage},
};

/// A helper structure for people that manually implement [TableAssociations]
#[allow(
  // `Table` derives `Debug` but for some reason such thing is not allowed here
  missing_debug_implementations
)]
pub struct TableAssociationWrapper<'entity, T, TS>
where
  T: Table<'entity>,
  TS: Lease<[TableParams<'entity, T>]> + SingleTypeStorage<Item = TableParams<'entity, T>>,
{
  /// See [TableAssociation]
  pub association: TableAssociation,
  /// Used to construct SELECT operations
  pub guide: TableParams<'entity, T>,
  /// A storage of zero, one or many tables used for INSERT and UPDATE operations
  pub tables: TS,
}
