use crate::{
  collection::Vector,
  database::{
    Database, Identifier,
    schema_manager::{
      Commands, DbMigration, MigrationStatus, SchemaManagement, integration_tests::AuxTestParams,
    },
  },
};
use alloc::string::String;

pub(crate) async fn _migrate_works<DB, E>(
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
  crate::database::schema_manager::integration_tests::schema::migrate_works(buffer_cmd, c, aux, 6)
    .await
}
