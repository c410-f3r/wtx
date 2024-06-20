use crate::database::{
  orm::{
    node_was_already_visited, table_fields::TableFields, AuxNodes, SqlValue, SqlWriter,
    SqlWriterLogic, Table, TableParams,
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
  pub(crate) async fn write_delete<EX>(
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
    table: &mut TableParams<'entity, T>,
  ) -> Result<(), <T::Database as Database>::Error>
  where
    EX: Executor<Database = T::Database>,
  {
    if node_was_already_visited(aux, table)? {
      return Ok(());
    }
    table.associations_mut().write_delete(aux, buffer_cmd, executor).await?;
    let id_value = if let Some(el) = table.fields().id().value() { el } else { return Ok(()) };
    let before = buffer_cmd.len();
    buffer_cmd
      .write_fmt(format_args!(
        "DELETE FROM {} WHERE {}=",
        T::TABLE_NAME,
        table.fields().id().name()
      ))
      .map_err(From::from)?;
    id_value.write(buffer_cmd)?;
    buffer_cmd.push(';');
    executor.execute(buffer_cmd.get(before..).unwrap_or_default(), |_| {}).await?;
    Ok(())
  }
}
