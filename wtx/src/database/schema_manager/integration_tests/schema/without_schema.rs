use crate::{
  database::{
    Identifier,
    schema_manager::{
      Commands, DbMigration, MigrationStatus, SchemaManagement, integration_tests::AuxTestParams,
    },
  },
  misc::Vector,
};
use alloc::string::String;

pub(crate) async fn _migrate_works<E>(
  (buffer_cmd, _, _, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
    &mut Vector<MigrationStatus>,
  ),
  c: &mut Commands<E>,
  aux: AuxTestParams,
) where
  E: SchemaManagement,
{
  crate::database::schema_manager::integration_tests::schema::migrate_works(buffer_cmd, c, aux, 6)
    .await
}
