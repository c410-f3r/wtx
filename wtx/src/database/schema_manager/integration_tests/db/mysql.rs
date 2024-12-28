#[cfg(feature = "schema-manager-dev")]
use {
  crate::database::{
    client::mysql::Mysql, schema_manager::integration_tests, schema_manager::Commands,
    schema_manager::DbMigration, schema_manager::SchemaManagement, FromRecord, Identifier,
  },
  crate::misc::Vector,
  alloc::string::String,
};

#[cfg(feature = "schema-manager-dev")]
pub(crate) async fn _clean_drops_all_objs<E>(
  (buffer_cmd, _, buffer_idents): (&mut String, &mut Vector<DbMigration>, &mut Vector<Identifier>),
  c: &mut Commands<E>,
  _: integration_tests::AuxTestParams,
) where
  E: SchemaManagement<Database = Mysql<crate::Error>>,
  Identifier: FromRecord<Mysql<crate::Error>>,
{
  integration_tests::create_foo_table(buffer_cmd, c, "").await;
  c.executor.table_names(buffer_cmd, buffer_idents, "").await.unwrap();
  assert_eq!(buffer_idents.len(), 1);

  c.clear((buffer_cmd, buffer_idents)).await.unwrap();

  c.executor.table_names(buffer_cmd, buffer_idents, "").await.unwrap();
  assert_eq!(buffer_idents.len(), 0);
}
