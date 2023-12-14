use crate::database::{
  sm::{integration_tests::AuxTestParams, Commands, DbMigration, SchemaManagement},
  Identifier,
};

pub(crate) async fn _migrate_works<E>(
  (buffer_cmd, _, _): (&mut String, &mut Vec<DbMigration>, &mut Vec<Identifier>),
  c: &mut Commands<E>,
  aux: AuxTestParams,
) where
  E: SchemaManagement,
{
  crate::database::sm::integration_tests::schema::migrate_works(buffer_cmd, c, aux, 6).await
}
