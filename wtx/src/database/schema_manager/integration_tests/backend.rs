use crate::{
  collection::Vector,
  database::{
    Identifier,
    schema_manager::{
      Commands, DbMigration, SchemaManagement,
      integration_tests::{_migrate_doc_test, AuxTestParams},
    },
  },
  time::{DateTime, Instant},
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
  let created_on = buffer_db_migrations[0].created_on().timestamp().0;
  let range = created_on..=created_on + 5;
  let now =
    DateTime::from_timestamp_secs(Instant::now_timestamp().unwrap().as_secs().cast_signed())
      .unwrap()
      .timestamp()
      .0;
  assert!(range.contains(&now));
}
