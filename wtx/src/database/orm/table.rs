use crate::{
  database::{
    orm::{FromSuffixRslt, TableAssociations, TableFields, TableParams},
    Database, RecordValues, TableSuffix,
  },
  misc::AsyncBounds,
};

/// All SQL definitions of an entity table.
pub trait Table<'entity>: Sized {
  /// Table name specified in the database
  const TABLE_NAME: &'static str;
  /// Optional table alias specified in the database
  const TABLE_NAME_ALIAS: Option<&'static str> = None;

  /// See [TableAssociations]
  type Associations: TableAssociations;
  /// See [Database].
  type Database: Database;
  /// All table fields minus the primary key. For more information, see [TableFields]
  type Fields: AsyncBounds + RecordValues<Self::Database> + TableFields<Self::Database>;

  /// Implementation should provide all related fields and associations
  fn type_instances(ts: TableSuffix) -> FromSuffixRslt<'entity, Self>;

  /// Updates the inner instance values that are used by some CRUD operations
  fn update_all_table_fields(&'entity self, tp: &mut TableParams<'entity, Self>);
}

impl<'entity> Table<'entity> for () {
  const TABLE_NAME: &'static str = "";

  type Associations = ();
  type Database = ();
  type Fields = ();

  #[inline]
  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    ((), ())
  }

  #[inline]
  fn update_all_table_fields(&'entity self, _: &mut TableParams<'entity, Self::Fields>) {}
}
