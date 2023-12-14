use crate::database::{
  sm::{
    integration_tests::{AuxTestParams, _migrate_doc_test},
    Commands, DbMigration, SchemaManagement,
  },
  Identifier,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};

pub(crate) async fn _backend_has_migration_with_utc_time<E>(
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
  let mg = _migrate_doc_test((buffer_cmd, buffer_db_migrations, buffer_idents), c).await;
  c.executor.migrations(buffer_cmd, &mg, buffer_db_migrations).await.unwrap();
  let created_on = *buffer_db_migrations[0].created_on();
  let range = created_on..=created_on + Duration::seconds(5);
  let utc: DateTime<FixedOffset> = Utc::now().into();
  assert!(range.contains(&utc));
}
