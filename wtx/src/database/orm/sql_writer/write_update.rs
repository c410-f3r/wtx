use crate::database::{
  orm::{
    node_was_already_visited, truncate_if_ends_with_char, AuxNodes, SqlValue, SqlWriter,
    SqlWriterLogic, Table, TableFields, TableParams,
  },
  Database, Executor,
};
use alloc::string::String;
use core::fmt::Write;

impl<'entity, T> SqlWriterLogic<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<T::Database>,
{
  #[inline]
  pub(crate) async fn write_update<EX>(
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
    table: &mut TableParams<'entity, T>,
  ) -> Result<(), <T::Database as Database>::Error>
  where
    EX: Executor,
  {
    if node_was_already_visited(aux, table)? {
      return Ok(());
    }

    let id_value = if let Some(el) = table.fields().id().value() { el } else { return Ok(()) };
    buffer_cmd.write_fmt(format_args!("UPDATE {} SET ", T::TABLE_NAME)).map_err(From::from)?;
    table.fields().write_update_values(buffer_cmd)?;
    truncate_if_ends_with_char(buffer_cmd, ',');
    buffer_cmd.push_str(" WHERE ");
    buffer_cmd.write_fmt(format_args!("{}=", table.fields().id().name())).map_err(From::from)?;
    id_value.write(buffer_cmd)?;
    buffer_cmd.push(';');

    table.associations_mut().write_update(aux, buffer_cmd, executor).await?;

    executor.execute(buffer_cmd, |_| {}).await?;
    Ok(())
  }
}
