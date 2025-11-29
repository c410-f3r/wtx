use crate::{
  collection::Vector,
  database::{
    Database, Identifier,
    schema_manager::{
      self, Commands, DbMigration, MigrationStatus, SchemaManagement,
      integration_tests::{_migrate_doc_test, AuxTestParams},
    },
  },
};
use alloc::string::String;

pub(crate) async fn all_tables_returns_the_number_of_tables_of_wtx_schema<DB, E>(
  (buffer_cmd, _, buffer_idents, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
    &mut Vector<MigrationStatus>,
  ),
  c: &mut Commands<E>,
  _: AuxTestParams,
) where
  DB: Database<Error = crate::Error>,
  E: SchemaManagement<Database = DB>,
{
  c.executor_mut().table_names(buffer_cmd, buffer_idents, "_wtx").await.unwrap();
  assert_eq!(buffer_idents.len(), 0);
  let _ = _migrate_doc_test(c).await;

  c.executor_mut().table_names(buffer_cmd, buffer_idents, "_wtx").await.unwrap();
  assert_eq!(buffer_idents.len(), 2);
  buffer_idents.clear();
}

pub(crate) async fn migrate_works<DB, E>(
  (buffer_cmd, _, _, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
    &mut Vector<MigrationStatus>,
  ),
  c: &mut Commands<E>,
  aux: AuxTestParams,
) where
  DB: Database<Error = crate::Error>,
  E: SchemaManagement<Database = DB>,
{
  schema_manager::integration_tests::schema::migrate_works(buffer_cmd, c, aux, 2).await
}
