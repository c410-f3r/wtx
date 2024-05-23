use crate::database::{
  orm::{
    node_was_already_visited, truncate_if_ends_with_char, AuxNodes, SqlWriter, SqlWriterLogic,
    Table, TableFields, TableParams,
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
  pub(crate) fn write_insert(
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
    tsa: &mut Option<&'static str>,
  ) -> Result<(), <T::Database as Database>::Error> {
    if node_was_already_visited(aux, table)? {
      return Ok(());
    }

    let elem_opt = || {
      if let Some(el) = *tsa {
        (el != table.id_field().name()).then_some(el)
      } else {
        None
      }
    };

    if let Some(elem) = elem_opt() {
      let bind_prefix = <T::Database as Database>::BIND_PREFIX;
      Self::write_insert_manager(
        buffer_cmd,
        table,
        |local| local.write_fmt(format_args!(",{}", elem)).map_err(From::from),
        |idx, local| {
          if <T::Database as Database>::IS_BIND_INCREASING {
            local.write_fmt(format_args!(",{bind_prefix}{idx}")).map_err(From::from)
          } else {
            local.write_fmt(format_args!(",{bind_prefix}")).map_err(From::from)
          }
        },
      )?;
    } else {
      Self::write_insert_manager(buffer_cmd, table, |_| Ok(()), |_, _| Ok(()))?;
    }

    table.associations().write_insert(aux, buffer_cmd, &mut Some(T::PRIMARY_KEY_NAME))?;

    Ok(())
  }

  fn write_insert_manager(
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
    foreign_key_name_cb: impl Fn(&mut String) -> crate::Result<()>,
    foreign_key_value_cb: impl Fn(usize, &mut String) -> crate::Result<()>,
  ) -> Result<(), <T::Database as Database>::Error> {
    let len_before_insert = buffer_cmd.len();
    let bind_prefix = <T::Database as Database>::BIND_PREFIX;

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
    if table.id_field().value().is_some() {
      if <T::Database as Database>::IS_BIND_INCREASING {
        buffer_cmd.push_str(bind_prefix);
        buffer_cmd.push('1');
      } else {
        buffer_cmd.push_str(bind_prefix);
      }
    }

    let iter = table.fields().opt_fields().filter(|elem| !*elem);
    let mut idx: usize = 2;
    if <T::Database as Database>::IS_BIND_INCREASING {
      for _ in iter {
        buffer_cmd.write_fmt(format_args!(",{bind_prefix}{idx}")).map_err(From::from)?;
        idx = idx.wrapping_add(1);
      }
    } else {
      for _ in iter {
        buffer_cmd.push(',');
        buffer_cmd.push_str(bind_prefix);
      }
    }

    if buffer_cmd.len() == len_before_values {
      buffer_cmd.truncate(len_before_insert);
    } else {
      foreign_key_value_cb(idx, buffer_cmd)?;
      truncate_if_ends_with_char(buffer_cmd, ',');
      buffer_cmd.push_str(");");
    }
    Ok(())
  }
}
