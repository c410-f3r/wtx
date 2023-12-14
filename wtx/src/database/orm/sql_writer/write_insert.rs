use crate::database::orm::{
  node_was_already_visited, truncate_if_ends_with_char, AuxNodes, SqlValue, SqlWriter,
  SqlWriterLogic, Table, TableFields, TableParams, TableSourceAssociation,
};
use alloc::string::String;
use core::fmt::{Display, Write};

impl<'entity, T> SqlWriterLogic<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<Error = T::Error>,
{
  #[inline]
  pub(crate) fn write_insert<V>(
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
    tsa: &mut Option<TableSourceAssociation<'_, V>>,
  ) -> Result<(), T::Error>
  where
    V: Display,
  {
    if node_was_already_visited(aux, table)? {
      return Ok(());
    }

    let elem_opt = || {
      if let Some(ref el) = *tsa {
        (el.source_field() != table.id_field().name()).then_some(el)
      } else {
        None
      }
    };

    if let Some(elem) = elem_opt() {
      Self::write_insert_manager(
        buffer_cmd,
        table,
        |local| local.write_fmt(format_args!(",{}", elem.source_field())).map_err(From::from),
        |local| local.write_fmt(format_args!("'{}',", elem.source_value())).map_err(From::from),
      )?;
    } else {
      Self::write_insert_manager(buffer_cmd, table, |_| Ok(()), |_| Ok(()))?;
    }

    let mut new_tsa = table.id_field().value().as_ref().map(TableSourceAssociation::new);
    table.associations().write_insert(aux, buffer_cmd, &mut new_tsa)?;

    Ok(())
  }

  fn write_insert_manager(
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
    foreign_key_name_cb: impl Fn(&mut String) -> crate::Result<()>,
    foreign_key_value_cb: impl Fn(&mut String) -> crate::Result<()>,
  ) -> Result<(), T::Error> {
    let len_before_insert = buffer_cmd.len();

    buffer_cmd
      .write_fmt(format_args!("INSERT INTO \"{}\" (", T::TABLE_NAME))
      .map_err(From::from)?;
    buffer_cmd.push_str(table.id_field().name());
    for field in table.fields().field_names() {
      buffer_cmd.write_fmt(format_args!(",{field}")).map_err(From::from)?;
    }
    foreign_key_name_cb(buffer_cmd)?;

    buffer_cmd.push_str(") VALUES (");
    let len_before_values = buffer_cmd.len();
    if let &Some(elem) = table.id_field().value() {
      elem.write(buffer_cmd)?;
      buffer_cmd.push(',');
    }
    table.fields().write_insert_values(buffer_cmd)?;

    if buffer_cmd.len() == len_before_values {
      buffer_cmd.truncate(len_before_insert);
    } else {
      foreign_key_value_cb(buffer_cmd)?;
      truncate_if_ends_with_char(buffer_cmd, ',');
      buffer_cmd.push_str(");");
    }
    Ok(())
  }
}
