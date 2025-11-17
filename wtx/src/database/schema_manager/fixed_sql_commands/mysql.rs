use crate::{
  collection::Vector,
  database::{Executor, FromRecords, Identifier, client::mysql::Mysql},
};

pub(crate) const CREATE_MIGRATION_TABLES: &str = concat!(
  "CREATE TABLE IF NOT EXISTS _wtx_migration_group (",
  _wtx_migration_group_columns!(),
  ");
  CREATE TABLE IF NOT EXISTS _wtx_migration (",
  _serial_id!(),
  "created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,",
  _wtx_migration_columns!(),
  ");"
);

// https://stackoverflow.com/questions/12403662/how-to-remove-all-mysql-tables-from-the-command-line-without-drop-database-permi/18625545#18625545
pub(crate) async fn clear<E>(executor: &mut E) -> crate::Result<()>
where
  E: Executor<Database = Mysql<crate::Error>>,
{
  let cmd = "
    SET FOREIGN_KEY_CHECKS = 0;
    SET GROUP_CONCAT_MAX_LEN=32768;
    SET @tables = NULL;
    SELECT GROUP_CONCAT('`', table_name, '`') INTO @tables
      FROM information_schema.tables
      WHERE table_schema = (SELECT DATABASE());
    SELECT IFNULL(@tables,'dummy') INTO @tables;

    SET @tables = CONCAT('DROP TABLE IF EXISTS ', @tables);
    PREPARE stmt FROM @tables;
    EXECUTE stmt;
    DEALLOCATE PREPARE stmt;
    SET FOREIGN_KEY_CHECKS = 1;
  ";
  executor.execute_many(&mut (), cmd, |_| Ok(())).await?;
  Ok(())
}

// https://github.com/flyway/flyway/blob/master/flyway-core/src/main/java/org/flywaydb/core/internal/database/mysql/MySQLSchema.java
pub(crate) async fn table_names<E>(
  executor: &mut E,
  results: &mut Vector<Identifier>,
) -> crate::Result<()>
where
  E: Executor<Database = Mysql<crate::Error>>,
{
  let cmd = "SELECT
      all_tables.table_name AS table_name
    FROM
      information_schema.tables AS all_tables
    WHERE
      all_tables.table_schema NOT IN ('performance_schema') AND all_tables.table_type IN ('BASE TABLE', 'SYSTEM VERSIONED')";
  let records = executor.execute_with_stmt_many(cmd, (), |_| Ok(())).await?;
  for elem in <Identifier as FromRecords<E::Database>>::many(&records) {
    results.push(elem?)?;
  }
  Ok(())
}
