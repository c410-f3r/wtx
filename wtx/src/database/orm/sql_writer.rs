mod write_delete;
mod write_insert;
mod write_select;
mod write_update;

use crate::database::{
  orm::{
    write_full_select_field, write_select_join, write_select_order_by, AuxNodes, SelectLimit,
    SelectOrderBy, Table, TableAssociations, TableFields, TableParams,
  },
  Database, Executor,
};
use alloc::string::String;
use core::{future::Future, marker::PhantomData};

/// Writes raw SQL commands
pub trait SqlWriter<DB>
where
  DB: Database,
{
  /// Writes an entire DELETE command
  fn write_delete<EX>(
    &mut self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
  ) -> impl Future<Output = Result<(), DB::Error>>
  where
    EX: Executor<Database = DB>;

  /// Writes an entire INSERT command
  fn write_insert<EX>(
    &mut self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
    params: (bool, Option<(&'static str, u64)>),
  ) -> impl Future<Output = Result<(), DB::Error>>
  where
    EX: Executor<Database = DB>;

  /// Writes an entire SELECT command
  fn write_select(
    &self,
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    limit: SelectLimit,
    where_cb: &mut impl FnMut(&mut String) -> Result<(), DB::Error>,
  ) -> Result<(), DB::Error>;

  /// Only writes JOIN commands that belong to SELECT
  fn write_select_associations(&self, buffer_cmd: &mut String) -> Result<(), DB::Error>;

  /// Only writes querying fields that belong to SELECT
  fn write_select_fields(&self, buffer_cmd: &mut String) -> Result<(), DB::Error>;

  /// Only writes ORDER BY commands that belong to SELECT
  fn write_select_orders_by(&self, buffer_cmd: &mut String) -> Result<(), DB::Error>;

  /// Writes an entire UPDATE command
  fn write_update<EX>(
    &mut self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
  ) -> impl Future<Output = Result<(), DB::Error>>
  where
    EX: Executor;
}

impl<DB> SqlWriter<DB> for ()
where
  DB: Database,
{
  #[inline]
  async fn write_delete<EX>(
    &mut self,
    _: &mut AuxNodes,
    _: &mut String,
    _: &mut EX,
  ) -> Result<(), DB::Error>
  where
    EX: Executor<Database = DB>,
  {
    Ok(())
  }

  #[inline]
  async fn write_insert<EX>(
    &mut self,
    _: &mut AuxNodes,
    _: &mut String,
    _: &mut EX,
    _: (bool, Option<(&'static str, u64)>),
  ) -> Result<(), DB::Error>
  where
    EX: Executor<Database = DB>,
  {
    Ok(())
  }

  #[inline]
  fn write_select(
    &self,
    _: &mut String,
    _: SelectOrderBy,
    _: SelectLimit,
    _: &mut impl FnMut(&mut String) -> Result<(), DB::Error>,
  ) -> Result<(), DB::Error> {
    Ok(())
  }

  #[inline]
  fn write_select_associations(&self, _: &mut String) -> Result<(), DB::Error> {
    Ok(())
  }

  #[inline]
  fn write_select_fields(&self, _: &mut String) -> Result<(), DB::Error> {
    Ok(())
  }

  #[inline]
  fn write_select_orders_by(&self, _: &mut String) -> Result<(), DB::Error> {
    Ok(())
  }

  #[inline]
  async fn write_update<EX>(
    &mut self,
    _: &mut AuxNodes,
    _: &mut String,
    _: &mut EX,
  ) -> Result<(), DB::Error>
  where
    EX: Executor,
  {
    Ok(())
  }
}

impl<'entity, T> SqlWriter<T::Database> for TableParams<'entity, T>
where
  T: Table<'entity>,
  T::Associations: SqlWriter<T::Database>,
{
  #[inline]
  async fn write_delete<EX>(
    &mut self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
  ) -> Result<(), <T::Database as Database>::Error>
  where
    EX: Executor<Database = T::Database>,
  {
    SqlWriterLogic::write_delete(aux, buffer_cmd, executor, self).await
  }

  #[inline]
  async fn write_insert<EX>(
    &mut self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
    params: (bool, Option<(&'static str, u64)>),
  ) -> Result<(), <T::Database as Database>::Error>
  where
    EX: Executor<Database = T::Database>,
  {
    SqlWriterLogic::write_insert(aux, buffer_cmd, executor, self, params).await
  }

  #[inline]
  fn write_select(
    &self,
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    select_limit: SelectLimit,
    where_cb: &mut impl FnMut(&mut String) -> Result<(), <T::Database as Database>::Error>,
  ) -> Result<(), <T::Database as Database>::Error> {
    SqlWriterLogic::write_select(buffer_cmd, order_by, select_limit, self, where_cb)
  }

  #[inline]
  fn write_select_associations(
    &self,
    buffer_cmd: &mut String,
  ) -> Result<(), <T::Database as Database>::Error> {
    for full_association in self.associations().full_associations() {
      write_select_join(buffer_cmd, T::TABLE_NAME, self.table_suffix(), full_association)?;
      buffer_cmd.push(' ');
    }
    self.associations().write_select_associations(buffer_cmd)?;
    Ok(())
  }

  #[inline]
  fn write_select_fields(
    &self,
    buffer_cmd: &mut String,
  ) -> Result<(), <T::Database as Database>::Error> {
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
  fn write_select_orders_by(
    &self,
    buffer_cmd: &mut String,
  ) -> Result<(), <T::Database as Database>::Error> {
    write_select_order_by(
      buffer_cmd,
      T::TABLE_NAME,
      T::TABLE_NAME_ALIAS,
      self.table_suffix(),
      self.fields().id().name(),
    )?;
    buffer_cmd.push(',');
    self.associations().write_select_orders_by(buffer_cmd)?;
    Ok(())
  }

  #[inline]
  async fn write_update<EX>(
    &mut self,
    aux: &mut AuxNodes,
    buffer_cmd: &mut String,
    executor: &mut EX,
  ) -> Result<(), <T::Database as Database>::Error>
  where
    EX: Executor,
  {
    SqlWriterLogic::write_update(aux, buffer_cmd, executor, self).await
  }
}

pub(crate) struct SqlWriterLogic<'entity, T>(PhantomData<(&'entity (), T)>)
where
  T: Table<'entity>,
  T::Associations: SqlWriter<T::Database>;
