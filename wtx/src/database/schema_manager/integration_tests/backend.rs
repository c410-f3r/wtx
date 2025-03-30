use crate::{
  database::{
    Identifier,
    schema_manager::{
      Commands, DbMigration, SchemaManagement,
      integration_tests::{_migrate_doc_test, AuxTestParams},
    },
  },
  misc::Vector,
};
use alloc::string::String;
use chrono::{DateTime, Duration, FixedOffset, Utc};

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
  c._executor_mut().migrations(buffer_cmd, &mg, buffer_db_migrations).await.unwrap();
  let created_on = *buffer_db_migrations[0].created_on();
  let range = created_on..=created_on + Duration::seconds(5);
  let utc: DateTime<FixedOffset> = Utc::now().into();
  assert!(range.contains(&utc));
}
