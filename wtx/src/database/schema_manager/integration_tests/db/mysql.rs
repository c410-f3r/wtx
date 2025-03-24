#[cfg(feature = "schema-manager-dev")]
use {
  crate::database::{
    FromRecord, Identifier, client::mysql::Mysql, schema_manager::Commands,
    schema_manager::DbMigration, schema_manager::MigrationStatus, schema_manager::SchemaManagement,
    schema_manager::integration_tests,
  },
  crate::misc::Vector,
  alloc::string::String,
};

#[cfg(feature = "schema-manager-dev")]
pub(crate) async fn _clean_drops_all_objs<E>(
  (buffer_cmd, _, buffer_idents, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
    &mut Vector<MigrationStatus>,
  ),
  c: &mut Commands<E>,
  _: integration_tests::AuxTestParams,
) where
  E: SchemaManagement<Database = Mysql<crate::Error>>,
  Identifier: FromRecord<Mysql<crate::Error>>,
{
  integration_tests::create_foo_table(buffer_cmd, c, "").await;

  c.executor.table_names(buffer_cmd, buffer_idents, "").await.unwrap();
  assert_eq!(buffer_idents.len(), 1);
  buffer_idents.clear();

  c.clear((buffer_cmd, buffer_idents)).await.unwrap();

  c.executor.table_names(buffer_cmd, buffer_idents, "").await.unwrap();
  assert_eq!(buffer_idents.len(), 0);
  buffer_idents.clear();
}
