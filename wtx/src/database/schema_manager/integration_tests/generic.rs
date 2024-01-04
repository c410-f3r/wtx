use crate::database::{
  schema_manager::{
    integration_tests::AuxTestParams, Commands, DbMigration, MigrationGroup, SchemaManagement,
  },
  Identifier,
};
use std::path::Path;

pub(crate) async fn all_tables_returns_the_number_of_tables_of_the_default_schema<E>(
  (buffer_cmd, _, buffer_idents): (&mut String, &mut Vec<DbMigration>, &mut Vec<Identifier>),
  c: &mut Commands<E>,
  aux: AuxTestParams,
) where
  E: SchemaManagement,
{
  c.executor.execute("CREATE TABLE IF NOT EXISTS foo(id INT)", |_| {}).await.unwrap();
  c.executor.table_names(buffer_cmd, buffer_idents, aux.default_schema).await.unwrap();
  assert_eq!(buffer_idents.len(), 1);
  buffer_idents.clear();
}

pub(crate) async fn rollback_works<E>(
  (buffer_cmd, buffer_db_migrations, buffer_idents): (
    &mut String,
    &mut Vec<DbMigration>,
    &mut Vec<Identifier>,
  ),
  c: &mut Commands<E>,
  aux: AuxTestParams,
) where
  E: SchemaManagement,
{
  let path = Path::new("../.test-utils/migrations.toml");
  c.migrate_from_toml_path((buffer_cmd, buffer_db_migrations), path).await.unwrap();
  c.rollback_from_toml((buffer_cmd, buffer_db_migrations), path, &[0, 0][..]).await.unwrap();
  let initial = MigrationGroup::new("initial", 1);
  let more_stuff = MigrationGroup::new("more_stuff", 2);

  c.executor.migrations(buffer_cmd, &initial, buffer_db_migrations).await.unwrap();
  assert_eq!(buffer_db_migrations.len(), 0);

  c.executor.migrations(buffer_cmd, &more_stuff, buffer_db_migrations).await.unwrap();
  assert_eq!(buffer_db_migrations.len(), 0);

  c.executor.table_names(buffer_cmd, buffer_idents, aux.default_schema).await.unwrap();
  assert_eq!(buffer_idents.len(), aux.schema_regulator);
  buffer_idents.clear();

  c.executor.table_names(buffer_cmd, buffer_idents, aux.wtx_schema).await.unwrap();
  assert_eq!(buffer_idents.len(), 2);
  buffer_idents.clear();
}
