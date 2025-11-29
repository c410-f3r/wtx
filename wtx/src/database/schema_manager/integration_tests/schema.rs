pub(crate) mod with_schema;
pub(crate) mod without_schema;

use crate::{
  collection::Vector,
  database::{
    Database,
    schema_manager::{
      Commands, SchemaManagement, UserMigrationGroup, integration_tests::AuxTestParams,
    },
  },
};
use alloc::string::String;
use std::path::Path;

pub(crate) async fn migrate_works<DB, E>(
  buffer_cmd: &mut String,
  c: &mut Commands<E>,
  aux: AuxTestParams,
  wtx_schema_tables: usize,
) where
  DB: Database<Error = crate::Error>,
  E: SchemaManagement<Database = DB>,
{
  let path = Path::new("../.test-utils/migrations.toml");
  let mut db_migrations = Vector::new();
  c.migrate_from_toml_path(path).await.unwrap();
  let initial = UserMigrationGroup::new("initial", 1);
  c.executor_mut().migrations(buffer_cmd, &initial, &mut db_migrations).await.unwrap();
  assert_eq!(db_migrations[0].checksum(), 7573493478190316387);
  assert_eq!(db_migrations[0].uid(), 1);
  assert_eq!(db_migrations[0].name(), "create_author");
  assert_eq!(db_migrations[1].uid(), 2);
  assert_eq!(db_migrations[1].name(), "create_post");
  assert_eq!(db_migrations[2].uid(), 3);
  assert_eq!(db_migrations[2].name(), "insert_author");
  assert_eq!(db_migrations[3].uid(), 4);
  assert_eq!(db_migrations[3].name(), "insert_post");
  assert_eq!(db_migrations.get(4), None);
  let more_stuff = UserMigrationGroup::new("more_stuff", 2);
  db_migrations.clear();
  c.executor_mut().migrations(buffer_cmd, &more_stuff, &mut db_migrations).await.unwrap();
  assert_eq!(db_migrations[0].checksum(), 8208328219135761847);
  assert_eq!(db_migrations[0].uid(), 1);
  assert_eq!(db_migrations[0].name(), "create_stuff");
  assert_eq!(db_migrations[1].uid(), 2);
  assert_eq!(db_migrations[1].name(), "insert_stuff");
  assert_eq!(db_migrations.get(4), None);
  let mut idents = Vector::new();
  c.executor_mut().table_names(buffer_cmd, &mut idents, aux.default_schema).await.unwrap();
  assert_eq!(idents.len(), 4 + aux.schema_regulator);
  idents.clear();
  c.executor_mut().table_names(buffer_cmd, &mut idents, aux.wtx_schema).await.unwrap();
  assert_eq!(idents.len(), wtx_schema_tables);
}
