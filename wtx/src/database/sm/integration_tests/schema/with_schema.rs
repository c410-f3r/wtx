use crate::database::{
  sm::{
    integration_tests::{AuxTestParams, _migrate_doc_test},
    Commands, DbMigration, SchemaManagement,
  },
  Identifier,
};

pub(crate) async fn all_tables_returns_the_number_of_tables_of_wtx_schema<E>(
  (buffer_cmd, buffer_db_migrations, buffer_idents): (
    &mut String,
    &mut Vec<DbMigration>,
    &mut Vec<Identifier>,
  ),
  c: &mut Commands<E>,
  _: AuxTestParams,
) where
  E: SchemaManagement,
{
  c.executor.table_names(buffer_cmd, buffer_idents, "_wtx").await.unwrap();
  assert_eq!(buffer_idents.len(), 0);
  let _ = _migrate_doc_test((buffer_cmd, buffer_db_migrations, buffer_idents), c).await;

  c.executor.table_names(buffer_cmd, buffer_idents, "_wtx").await.unwrap();
  assert_eq!(buffer_idents.len(), 2);
  buffer_idents.clear();
}

pub(crate) async fn migrate_works<E>(
  (buffer_cmd, _, _): (&mut String, &mut Vec<DbMigration>, &mut Vec<Identifier>),
  c: &mut Commands<E>,
  aux: AuxTestParams,
) where
  E: SchemaManagement,
{
  crate::database::sm::integration_tests::schema::migrate_works(buffer_cmd, c, aux, 2).await
}
