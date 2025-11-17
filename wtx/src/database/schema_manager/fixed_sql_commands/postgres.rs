use crate::{
  collection::Vector,
  database::{FromRecords, Identifier, client::postgres::Postgres, executor::Executor},
};
use alloc::string::String;
use core::fmt::Write;

pub(crate) static CREATE_MIGRATION_TABLES: &str = concat!(
  "CREATE SCHEMA IF NOT EXISTS _wtx; \
  CREATE TABLE IF NOT EXISTS _wtx._wtx_migration_group (",
  _wtx_migration_group_columns!(),
  ");
  CREATE TABLE IF NOT EXISTS _wtx._wtx_migration (",
  _serial_id!(),
  "created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,",
  _wtx_migration_columns!(),
  ");"
);

pub(crate) async fn all_elements<E>(
  (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
  executor: &mut E,
  schemas_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  sequences_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  domains_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  functions_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  views_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  table_names_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  procedures_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
  types_cb: impl FnOnce((&mut String, &mut Vector<Identifier>)) -> crate::Result<()>,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  schemas(executor, buffer_idents).await?;
  schemas_cb((buffer_cmd, buffer_idents))?;

  sequences(executor, buffer_idents).await?;
  sequences_cb((buffer_cmd, buffer_idents))?;

  domains(executor, buffer_idents).await?;
  domains_cb((buffer_cmd, buffer_idents))?;

  functions((buffer_cmd, buffer_idents), executor).await?;
  functions_cb((buffer_cmd, buffer_idents))?;

  views(executor, buffer_idents).await?;
  views_cb((buffer_cmd, buffer_idents))?;

  table_names(buffer_cmd, executor, buffer_idents, "public").await?;
  table_names_cb((buffer_cmd, buffer_idents))?;

  procedures((buffer_cmd, buffer_idents), executor).await?;
  procedures_cb((buffer_cmd, buffer_idents))?;

  types(executor, buffer_idents).await?;
  types_cb((buffer_cmd, buffer_idents))?;

  Ok(())
}

pub(crate) async fn clear<E>(
  (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
  executor: &mut E,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  all_elements(
    (buffer_cmd, buffer_idents),
    executor,
    |buffer| push_drop(buffer, "SCHEMA"),
    |buffer| push_drop(buffer, "SEQUENCE"),
    |buffer| push_drop(buffer, "DOMAIN"),
    |buffer| push_drop(buffer, "FUNCTION"),
    |buffer| push_drop(buffer, "VIEW"),
    |buffer| push_drop(buffer, "TABLE"),
    |buffer| push_drop(buffer, "PROCEDURE"),
    |buffer| push_drop(buffer, "TYPE"),
  )
  .await?;
  executor
    .transaction(|this| async {
      this.execute_ignored(buffer_cmd.as_str()).await?;
      Ok(((), this))
    })
    .await?;
  buffer_cmd.clear();
  Ok(())
}

pub(crate) async fn domains<E>(
  executor: &mut E,
  results: &mut Vector<Identifier>,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let cmd = "SELECT
      t.typname AS generic_column
    FROM pg_catalog.pg_type t
      LEFT JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
      LEFT JOIN pg_depend dep ON dep.objid = t.oid AND dep.deptype = 'e'
    WHERE t.typtype = 'd'
      AND n.nspname = 'public'
      AND dep.objid IS NULL";
  let records = executor.execute_stmt_many(cmd, (), |_| Ok(())).await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    results.push(elem?)?;
  }
  Ok(())
}

pub(crate) async fn functions<E>(
  (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
  executor: &mut E,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  pg_proc((buffer_cmd, buffer_idents), executor, 'f').await?;
  Ok(())
}

pub(crate) async fn procedures<E>(
  (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
  executor: &mut E,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  pg_proc((buffer_cmd, buffer_idents), executor, 'p').await?;
  Ok(())
}

pub(crate) async fn sequences<E>(
  executor: &mut E,
  results: &mut Vector<Identifier>,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let cmd = "SELECT
      sequence_name AS generic_column
    FROM
      information_schema.sequences
    WHERE
      sequence_schema = 'public'";
  let records = executor.execute_stmt_many(cmd, (), |_| Ok(())).await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    results.push(elem?)?;
  }
  Ok(())
}

pub(crate) async fn schemas<E>(
  executor: &mut E,
  results: &mut Vector<Identifier>,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let cmd = "SELECT
    pc_ns.nspname AS generic_column
  FROM
    pg_catalog.pg_namespace pc_ns
  WHERE
    nspname NOT IN ('information_schema', 'pg_catalog', 'public')
    AND nspname NOT LIKE 'pg_toast%'
    AND nspname NOT LIKE 'pg_temp_%'";
  let records = executor.execute_stmt_many(cmd, (), |_| Ok(())).await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    results.push(elem?)?;
  }
  Ok(())
}

pub(crate) async fn table_names<E>(
  buffer_cmd: &mut String,
  executor: &mut E,
  results: &mut Vector<Identifier>,
  schema: &str,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let before = buffer_cmd.len();
  buffer_cmd.write_fmt(format_args!(
    "SELECT
      tables.table_name AS generic_column
    FROM
      information_schema.tables tables
      -- that don't depend on an extension
      LEFT JOIN pg_depend dep ON dep.objid = (quote_ident(tables.table_schema)||'.'||quote_ident(tables.table_name))::regclass::oid AND dep.deptype = 'e'
    WHERE
      -- in this schema
      table_schema = '{schema}'
      -- that are real tables (as opposed to views)
      AND table_type='BASE TABLE'
      -- with no extension depending on them
      AND dep.objid IS NULL
      -- and are not child tables (= do not inherit from another table).
      AND NOT (
        SELECT EXISTS (SELECT inhrelid FROM pg_catalog.pg_inherits
        WHERE inhrelid = (quote_ident(tables.table_schema)||'.'||quote_ident(tables.table_name))::regclass::oid)
      )",
  ))?;
  let records = executor
    .execute_stmt_many(buffer_cmd.get(before..).unwrap_or_default(), (), |_| Ok(()))
    .await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    if let Err(elem) = results.push(elem?) {
      buffer_cmd.truncate(before);
      return Err(elem);
    }
  }
  buffer_cmd.truncate(before);
  Ok(())
}

pub(crate) async fn types<E>(
  executor: &mut E,
  results: &mut Vector<Identifier>,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let cmd = "SELECT
      typname AS generic_column
    FROM
      pg_catalog.pg_type t
      LEFT JOIN pg_depend dep ON dep.objid = t.oid and dep.deptype = 'e'
    WHERE
      (t.typrelid = 0 OR (
        SELECT c.relkind = 'c' FROM pg_catalog.pg_class c WHERE c.oid = t.typrelid)
      )
      AND NOT EXISTS(
        SELECT 1 FROM pg_catalog.pg_type el WHERE el.oid = t.typelem AND el.typarray = t.oid
      )
      AND t.typnamespace in (
        select oid from pg_catalog.pg_namespace where nspname = 'public'
      )
      AND dep.objid is null
      AND t.typtype != 'd'";
  let records = executor.execute_stmt_many(cmd, (), |_| Ok(())).await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    results.push(elem?)?;
  }
  Ok(())
}

pub(crate) async fn views<E>(
  executor: &mut E,
  results: &mut Vector<Identifier>,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let cmd = "SELECT
      relname AS generic_column
    FROM pg_catalog.pg_class c
      JOIN pg_namespace n ON n.oid = c.relnamespace
      LEFT JOIN pg_depend dep ON dep.objid = c.oid AND dep.deptype = 'e'
    WHERE c.relkind = 'v'
      AND  n.nspname = 'public'
      AND dep.objid IS NULL";
  let records = executor.execute_stmt_many(cmd, (), |_| Ok(())).await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    results.push(elem?)?;
  }
  Ok(())
}

async fn pg_proc<E>(
  (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
  executor: &mut E,
  prokind: char,
) -> crate::Result<()>
where
  E: Executor<Database = Postgres<crate::Error>>,
{
  let before = buffer_cmd.len();
  buffer_cmd.write_fmt(format_args!(
    "SELECT
      proname AS generic_column
    FROM
      pg_proc
      INNER JOIN pg_namespace ns ON (pg_proc.pronamespace = ns.oid)
      -- that don't depend on an extension
      LEFT JOIN pg_depend dep ON dep.objid = pg_proc.oid AND dep.deptype = 'e'
    WHERE
      ns.nspname = 'public'
      AND dep.objid IS NULL
      AND pg_proc.prokind = '{prokind}'",
  ))?;
  let records = executor
    .execute_stmt_many(buffer_cmd.get(before..).unwrap_or_default(), (), |_| Ok(()))
    .await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    if let Err(elem) = buffer_idents.push(elem?) {
      buffer_cmd.truncate(before);
      return Err(elem);
    }
  }
  buffer_cmd.truncate(before);
  Ok(())
}

fn push_drop(
  (buffer_cmd, buffer_idents): (&mut String, &mut Vector<Identifier>),
  structure: &str,
) -> crate::Result<()> {
  for identifier in buffer_idents.iter() {
    buffer_cmd.write_fmt(format_args!(r#"DROP {structure} "{identifier}" CASCADE;"#))?;
  }
  buffer_idents.clear();
  Ok(())
}
