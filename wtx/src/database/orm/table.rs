use crate::database::{
  orm::{FromSuffixRslt, SqlValue, TableAssociations, TableFields, TableParams},
  Database, TableSuffix,
};
use core::{fmt::Display, hash::Hash};

/// All SQL definitions of an entity table.
pub trait Table<'entity>: Sized {
  /// Table primary key name
  const PRIMARY_KEY_NAME: &'static str;
  /// Table name specified in the database
  const TABLE_NAME: &'static str;
  /// Optional table alias specified in the database
  const TABLE_NAME_ALIAS: Option<&'static str> = None;

  /// See [TableAssociations]
  type Associations: TableAssociations;
  /// See [Database].
  type Database: Database;
  /// All table fields minus the primary key. For more information, see [TableFields]
  type Fields: TableFields<Self::Database>;
  /// Table primary key value type
  type PrimaryKeyValue: Copy + Display + Hash + SqlValue<<Self::Database as Database>::Error>;

  /// Implementation should provide all related fields and associations
  fn type_instances(ts: TableSuffix) -> FromSuffixRslt<'entity, Self>;

  /// Updates the inner instance values that are used by some CRUD operations
  fn update_all_table_fields(entity: &'entity Self, table: &mut TableParams<'entity, Self>);
}

impl<'entity> Table<'entity> for () {
  const PRIMARY_KEY_NAME: &'static str = "";
  const TABLE_NAME: &'static str = "";

  type Associations = ();
  type Database = ();
  type Fields = ();
  type PrimaryKeyValue = &'static str;

  #[inline]
  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    ((), ())
  }

  #[inline]
  fn update_all_table_fields(_: &'entity Self, _: &mut TableParams<'entity, Self>) {}
}
