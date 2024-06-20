use crate::{
  database::{
    orm::{
      node_was_already_visited, table_associations::TableAssociations, truncate_if_ends_with_char,
      AuxNodes, SqlWriter, SqlWriterLogic, Table, TableFields, TableParams,
    },
    Database, Executor,
  },
  misc::_interspace,
};
use alloc::string::String;
use core::fmt::Write;

impl<'entity, T> SqlWriterLogic<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<T::Database>,
{
  #[inline]
  pub(crate) async fn write_insert<EX>(
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
    table: &mut TableParams<'entity, T>,
    (_, opt): (bool, Option<(&'static str, u64)>),
  ) -> Result<(), <T::Database as Database>::Error>
  where
    EX: Executor<Database = T::Database>,
  {
    if node_was_already_visited(aux, table)? {
      return Ok(());
    }
    table.associations_mut().write_insert(aux, buffer_cmd, executor, (true, None)).await?;
    let before = buffer_cmd.len();
    if let Some((name, value)) = opt {
      Self::write_insert_manager(
        buffer_cmd,
        table,
        |local| local.write_fmt(format_args!(",{name}")).map_err(From::from),
        |local| local.write_fmt(format_args!(",{value}")).map_err(From::from),
      )?;
    } else {
      Self::write_insert_manager(
        buffer_cmd,
        table,
        |local| {
          for elem in table.associations().full_associations() {
            if !elem.association().has_inverse_flow() {
              continue;
            }
            local.write_fmt(format_args!(",{}", elem.association().from_id_name()))?
          }
          Ok(())
        },
        |local| {
          for elem in table.associations().full_associations() {
            let has_inverse_flow = elem.association().has_inverse_flow();
            let (true, Some(id_value)) = (has_inverse_flow, elem.from_id_value()) else {
              continue;
            };
            local.write_fmt(format_args!(",{}", id_value))?;
          }
          Ok(())
        },
      )?;
    }
    let stmt = buffer_cmd.get(before..).unwrap_or_default();
    let _ = executor.execute_with_stmt(stmt, table.fields_mut()).await?;
    table.associations_mut().write_insert(aux, buffer_cmd, executor, (false, None)).await?;
    Ok(())
  }

  fn write_insert_manager(
    buffer_cmd: &mut String,
    table: &TableParams<'entity, T>,
    foreign_key_name_cb: impl Fn(&mut String) -> crate::Result<()>,
    foreign_key_value_cb: impl Fn(&mut String) -> crate::Result<()>,
  ) -> Result<(), <T::Database as Database>::Error> {
    let len_before_insert = buffer_cmd.len();
    let bind_prefix = <T::Database as Database>::BIND_PREFIX;

    buffer_cmd
      .write_fmt(format_args!("INSERT INTO \"{}\" (", T::TABLE_NAME))
      .map_err(From::from)?;
    _interspace(
      &mut *buffer_cmd,
      table.fields().field_names(),
      |local_buffer_cmd, elem| {
        local_buffer_cmd.write_fmt(format_args!("{elem}")).map_err(From::from)
      },
      |local_buffer_cmd| {
        local_buffer_cmd.push(',');
        Ok(())
      },
    )?;

    foreign_key_name_cb(buffer_cmd)?;

    buffer_cmd.push_str(") VALUES (");
    let len_before_values = buffer_cmd.len();
    let iter = table.fields().opt_fields().filter(|elem| !*elem);
    let mut idx: usize = 1;
    if <T::Database as Database>::IS_BIND_INCREASING {
      _interspace(
        &mut *buffer_cmd,
        iter,
        |local_buffer_cmd, _| {
          local_buffer_cmd.write_fmt(format_args!("{bind_prefix}{idx}"))?;
          idx = idx.wrapping_add(1);
          Ok(())
        },
        |local_buffer_cmd| {
          local_buffer_cmd.push(',');
          Ok(())
        },
      )?;
    } else {
      _interspace(
        &mut *buffer_cmd,
        iter,
        |local_buffer_cmd, _| {
          local_buffer_cmd.push_str(bind_prefix);
          idx = idx.wrapping_add(1);
          Ok(())
        },
        |local_buffer_cmd| {
          local_buffer_cmd.push(',');
          Ok(())
        },
      )?;
    }

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
