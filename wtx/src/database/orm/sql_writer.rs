mod write_delete;
mod write_insert;
mod write_select;
mod write_update;

use crate::database::orm::{
  write_full_select_field, write_select_join, write_select_order_by, AuxNodes, SelectLimit,
  SelectOrderBy, Table, TableAssociations, TableFields, TableParams, TableSourceAssociation,
};
use alloc::string::String;
use core::{fmt::Display, marker::PhantomData};

/// Writes raw SQL commands
pub trait SqlWriter {
  /// See [crate::Error].
  type Error: From<crate::Error>;

  /// Writes an entire DELETE command
  fn write_delete(&self, aux: &mut AuxNodes, buffer_cmd: &mut String) -> Result<(), Self::Error>;

  /// Writes an entire INSERT command
  fn write_insert<V>(
    &self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    table_source_association: &mut Option<TableSourceAssociation<'_, V>>,
  ) -> Result<(), Self::Error>
  where
    V: Display;

  /// Writes an entire SELECT command
  fn write_select(
    &self,
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    limit: SelectLimit,
    where_cb: &mut impl FnMut(&mut String) -> Result<(), Self::Error>,
  ) -> Result<(), Self::Error>;

  /// Only writes JOIN commands that belong to SELECT
  fn write_select_associations(&self, buffer_cmd: &mut String) -> Result<(), Self::Error>;

  /// Only writes querying fields that belong to SELECT
  fn write_select_fields(&self, buffer_cmd: &mut String) -> Result<(), Self::Error>;

  /// Only writes ORDER BY commands that belong to SELECT
  fn write_select_orders_by(&self, buffer_cmd: &mut String) -> Result<(), Self::Error>;

  /// Writes an entire UPDATE command
  fn write_update(&self, aux: &mut AuxNodes, buffer_cmd: &mut String) -> Result<(), Self::Error>;
}

impl<'entity, T> SqlWriter for TableParams<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<Error = T::Error>,
{
  type Error = T::Error;

  #[inline]
  fn write_delete(&self, aux: &mut AuxNodes, buffer_cmd: &mut String) -> Result<(), Self::Error> {
    SqlWriterLogic::write_delete(aux, buffer_cmd, self)
  }

  #[inline]
  fn write_insert<V>(
    &self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    tsa: &mut Option<TableSourceAssociation<'_, V>>,
  ) -> Result<(), Self::Error>
  where
    V: Display,
  {
    SqlWriterLogic::write_insert(aux, buffer_cmd, self, tsa)
  }

  #[inline]
  fn write_select(
    &self,
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    select_limit: SelectLimit,
    where_cb: &mut impl FnMut(&mut String) -> Result<(), Self::Error>,
  ) -> Result<(), Self::Error> {
    SqlWriterLogic::write_select(buffer_cmd, order_by, select_limit, self, where_cb)
  }

  #[inline]
  fn write_select_associations(&self, buffer_cmd: &mut String) -> Result<(), Self::Error> {
    for full_association in self.associations().full_associations() {
      write_select_join(buffer_cmd, T::TABLE_NAME, self.table_suffix(), full_association)?;
      buffer_cmd.push(' ');
    }
    self.associations().write_select_associations(buffer_cmd)?;
    Ok(())
  }

  #[inline]
  fn write_select_fields(&self, buffer_cmd: &mut String) -> Result<(), Self::Error> {
    write_full_select_field(
      buffer_cmd,
      T::TABLE_NAME,
      T::TABLE_NAME_ALIAS,
      self.table_suffix(),
      self.id_field().name(),
    )?;
    buffer_cmd.push(',');
    for field in self.fields().field_names() {
      write_full_select_field(
        buffer_cmd,
        T::TABLE_NAME,
        T::TABLE_NAME_ALIAS,
        self.table_suffix(),
        field,
      )?;
      buffer_cmd.push(',');
    }
    self.associations().write_select_fields(buffer_cmd)?;
    Ok(())
  }

  #[inline]
  fn write_select_orders_by(&self, buffer_cmd: &mut String) -> Result<(), Self::Error> {
    write_select_order_by(
      buffer_cmd,
      T::TABLE_NAME,
      T::TABLE_NAME_ALIAS,
      self.table_suffix(),
      self.id_field().name(),
    )?;
    buffer_cmd.push(',');
    self.associations().write_select_orders_by(buffer_cmd)?;
    Ok(())
  }

  #[inline]
  fn write_update(&self, aux: &mut AuxNodes, buffer_cmd: &mut String) -> Result<(), Self::Error> {
    SqlWriterLogic::write_update(aux, buffer_cmd, self)
  }
}

pub(crate) struct SqlWriterLogic<'entity, T>(PhantomData<(&'entity (), T)>)
where
  T: Table<'entity>,
  T::Associations: SqlWriter;
