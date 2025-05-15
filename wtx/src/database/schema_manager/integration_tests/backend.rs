use crate::{
  calendar::{Duration, Instant},
  collection::Vector,
  database::{
    Identifier,
    schema_manager::{
      Commands, DbMigration, SchemaManagement,
      integration_tests::{_migrate_doc_test, AuxTestParams},
    },
  },
};
use alloc::string::String;

pub(crate) async fn _backend_has_migration_with_utc_time<E>(
  (buffer_cmd, buffer_db_migrations, buffer_idents): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
  ),
  c: &mut Commands<E>,
  _: AuxTestParams,
) where
  E: SchemaManagement,
{
  let mg = _migrate_doc_test((buffer_cmd, buffer_db_migrations, buffer_idents), c).await;
  c.executor_mut().migrations(buffer_cmd, &mg, buffer_db_migrations).await.unwrap();
  let created_on = *buffer_db_migrations[0].created_on();
  let range = created_on..=created_on.add(Duration::from_seconds(5).unwrap()).unwrap();
  assert!(range.contains(&Instant::now_date_time(0).unwrap()));
}
