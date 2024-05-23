use crate::database::{
  orm::{
    node_was_already_visited, AuxNodes, SqlValue, SqlWriter, SqlWriterLogic, Table, TableParams,
  },
  Database,
};
use alloc::string::String;
use core::fmt::Write;

impl<'entity, T> SqlWriterLogic<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<Error = <T::Database as Database>::Error>,
{
  #[inline]
  pub(crate) fn write_delete(
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
  ) -> Result<(), <T::Database as Database>::Error> {
    if node_was_already_visited(aux, table)? {
      return Ok(());
    }
    table.associations().write_delete(aux, buffer_cmd)?;
    Self::write_delete_manager(buffer_cmd, table)?;
    Ok(())
  }

  fn write_delete_manager(
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
  ) -> Result<(), <T::Database as Database>::Error> {
    let id_value = if let Some(el) = table.id_field().value() { el } else { return Ok(()) };
    buffer_cmd
      .write_fmt(format_args!("DELETE FROM {} WHERE {}=", T::TABLE_NAME, T::PRIMARY_KEY_NAME))
      .map_err(From::from)?;
    id_value.write(buffer_cmd)?;
    buffer_cmd.push(';');
    Ok(())
  }
}
