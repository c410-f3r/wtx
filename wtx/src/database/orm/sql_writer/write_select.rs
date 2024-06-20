use crate::database::{
  orm::{
    truncate_if_ends_with_char, truncate_if_ends_with_str, SelectLimit, SelectOrderBy, SqlWriter,
    SqlWriterLogic, Table, TableParams,
  },
  Database,
};
use alloc::string::String;
use core::fmt::Write;

impl<'entity, T> SqlWriterLogic<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<T::Database>,
{
  #[inline]
  pub(crate) fn write_select(
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    select_limit: SelectLimit,
    table: &TableParams<'entity, T>,
    where_cb: &mut impl FnMut(&mut String) -> Result<(), <T::Database as Database>::Error>,
  ) -> Result<(), <T::Database as Database>::Error> {
    buffer_cmd.clear();
    buffer_cmd.push_str("SELECT ");
    table.write_select_fields(buffer_cmd)?;
    truncate_if_ends_with_char(buffer_cmd, ',');
    buffer_cmd
      .write_fmt(format_args!(
        " FROM \"{table}\" AS \"{table}{suffix}\" ",
        suffix = table.table_suffix(),
        table = T::TABLE_NAME
      ))
      .map_err(From::from)?;
    table.write_select_associations(buffer_cmd)?;
    buffer_cmd.push_str(" WHERE ");
    where_cb(buffer_cmd)?;
    truncate_if_ends_with_str(buffer_cmd, " WHERE ");
    buffer_cmd.push_str(" ORDER BY ");
    table.write_select_orders_by(buffer_cmd)?;
    truncate_if_ends_with_char(buffer_cmd, ',');
    match order_by {
      SelectOrderBy::Ascending => buffer_cmd.push_str(" ASC"),
      SelectOrderBy::Descending => buffer_cmd.push_str(" DESC"),
    }
    buffer_cmd.push_str(" LIMIT ");
    match select_limit {
      SelectLimit::All => buffer_cmd.push_str("ALL"),
      SelectLimit::Count(n) => {
        buffer_cmd.write_fmt(format_args!("{n}")).map_err(From::from)?;
      }
    }
    Ok(())
  }
}
