pub(crate) mod with_schema;
pub(crate) mod without_schema;

use crate::database::schema_manager::{
  integration_tests::AuxTestParams, Commands, MigrationGroup, SchemaManagement,
};
use std::path::Path;

pub(crate) async fn migrate_works<E>(
  buffer_cmd: &mut String,
  c: &mut Commands<E>,
  aux: AuxTestParams,
  wtx_schema_tables: usize,
) where
  E: SchemaManagement,
{
  let path = Path::new("../.test-utils/migrations.toml");
  let mut db_migrations = Vec::new();
  c.migrate_from_toml_path((buffer_cmd, &mut db_migrations), path).await.unwrap();
  let initial = MigrationGroup::new("initial", 1);
  c.executor.migrations(buffer_cmd, &initial, &mut db_migrations).await.unwrap();
  assert_eq!(db_migrations[0].checksum(), 7573493478190316387);
  assert_eq!(db_migrations[0].version(), 1);
  assert_eq!(db_migrations[0].name(), "create_author");
  assert_eq!(db_migrations[1].version(), 2);
  assert_eq!(db_migrations[1].name(), "create_post");
  assert_eq!(db_migrations[2].version(), 3);
  assert_eq!(db_migrations[2].name(), "insert_author");
  assert_eq!(db_migrations[3].version(), 4);
  assert_eq!(db_migrations[3].name(), "insert_post");
  assert_eq!(db_migrations.get(4), None);
  let more_stuff = MigrationGroup::new("more_stuff", 2);
  db_migrations.clear();
  c.executor.migrations(buffer_cmd, &more_stuff, &mut db_migrations).await.unwrap();
  assert_eq!(db_migrations[0].checksum(), 8208328219135761847);
  assert_eq!(db_migrations[0].version(), 1);
  assert_eq!(db_migrations[0].name(), "create_stuff");
  assert_eq!(db_migrations[1].version(), 2);
  assert_eq!(db_migrations[1].name(), "insert_stuff");
  assert_eq!(db_migrations.get(4), None);
  let mut idents = Vec::new();
  c.executor.table_names(buffer_cmd, &mut idents, aux.default_schema).await.unwrap();
  assert_eq!(idents.len(), 4 + aux.schema_regulator);
  idents.clear();
  c.executor.table_names(buffer_cmd, &mut idents, aux.wtx_schema).await.unwrap();
  assert_eq!(idents.len(), wtx_schema_tables);
}
