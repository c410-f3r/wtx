#[cfg(feature = "schema-manager-dev")]
use {
  crate::collection::Vector,
  crate::database::{
    FromRecords, Identifier, client::postgres::Postgres, schema_manager::Commands,
    schema_manager::DbMigration, schema_manager::MigrationStatus, schema_manager::SchemaManagement,
    schema_manager::fixed_sql_commands::postgres, schema_manager::integration_tests,
  },
  alloc::string::String,
};

#[cfg(feature = "schema-manager-dev")]
pub(crate) async fn clean_drops_all_objs<'exec, E>(
  (buffer_cmd, _, buffer_idents, _): (
    &mut String,
    &mut Vector<DbMigration>,
    &mut Vector<Identifier>,
    &mut Vector<MigrationStatus>,
  ),
  c: &'exec mut Commands<E>,
  _: integration_tests::AuxTestParams,
) where
  E: SchemaManagement<Database = Postgres<crate::Error>>,
  Identifier: FromRecords<'exec, Postgres<crate::Error>>,
{
  integration_tests::create_foo_table(buffer_cmd, c, "public.").await;
  c.executor_mut().execute_ignored("CREATE SCHEMA bar").await.unwrap();
  integration_tests::create_foo_table(buffer_cmd, c, "bar.").await;
  c.executor_mut().execute_ignored("CREATE DOMAIN integer0 AS INTEGER CONSTRAINT must_be_greater_than_or_equal_to_zero_chk CHECK(VALUE >= 0)").await.unwrap();
  c.executor_mut().execute_ignored("CREATE FUNCTION time_subtype_diff(x time, y time) RETURNS float8 AS 'SELECT EXTRACT(EPOCH FROM (x - y))' LANGUAGE sql STRICT IMMUTABLE").await.unwrap();
  c.executor_mut()
    .execute_ignored("CREATE PROCEDURE something() LANGUAGE SQL AS $$ $$")
    .await
    .unwrap();
  c.executor_mut().execute_ignored("CREATE SEQUENCE serial START 101").await.unwrap();
  c.executor_mut().execute_ignored("CREATE TYPE a_type AS (field INTEGER[31])").await.unwrap();
  c.executor_mut()
    .execute_ignored("CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy')")
    .await
    .unwrap();
  c.executor_mut()
    .execute_ignored("CREATE VIEW view AS SELECT * FROM foo WHERE id = 1")
    .await
    .unwrap();

  postgres::all_elements(
    (buffer_cmd, buffer_idents),
    &mut c.executor_mut(),
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 1);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 2);
      buffer.1.clear();
      Ok(())
    },
  )
  .await
  .unwrap();

  c.clear((buffer_cmd, buffer_idents)).await.unwrap();

  postgres::all_elements(
    (buffer_cmd, buffer_idents),
    c.executor_mut(),
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
    |buffer| {
      assert_eq!(buffer.1.len(), 0);
      buffer.1.clear();
      Ok(())
    },
  )
  .await
  .unwrap();
}
