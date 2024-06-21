use crate::{
  database::{
    executor::Executor,
    orm::{
      seek_related_entities, write_select_field, AuxNodes, SelectLimit, SelectOrderBy, SqlWriter,
      Table, TableFields, TableParams,
    },
    Database, Decode, FromRecords, Records, ValueIdent,
  },
  misc::AsyncBounds,
};
use alloc::string::String;
use core::{fmt::Write, future::Future};

/// Create, read, update and delete entities.
pub trait Crud: Executor
where
  Self: Sized,
{
  /// Inserts a new table record represented by `table_params`.
  fn create<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    table: &'entity T,
    table_params: &mut TableParams<'entity, T>,
  ) -> impl Future<Output = Result<(), <T::Database as Database>::Error>>
  where
    T: Table<'entity, Database = Self::Database>,
    T::Associations: SqlWriter<T::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut TableParams<'entity, T>: AsyncBounds,
    for<'any> &'any T: AsyncBounds,
  {
    async move {
      table_params.update_all_table_fields(table);
      table_params.write_insert(&mut AuxNodes::default(), buffer_cmd, self, (false, None)).await?;
      Ok(())
    }
  }

  /// Deletes all rows that compose a table.
  fn delete_all<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    table: &'entity T,
    table_params: &mut TableParams<'entity, T>,
  ) -> impl Future<Output = Result<(), <T::Database as Database>::Error>>
  where
    T: Table<'entity, Database = Self::Database>,
    T::Associations: SqlWriter<T::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut TableParams<'entity, T>: AsyncBounds,
    for<'any> &'any T: AsyncBounds,
  {
    async move {
      table_params.update_all_table_fields(table);
      table_params.write_delete(&mut AuxNodes::default(), buffer_cmd, self).await?;
      Ok(())
    }
  }

  /// Fetches all entities from the database.
  fn read_all<'entity, 'exec, T>(
    &'exec mut self,
    buffer_cmd: &mut String,
    tp: &TableParams<'entity, T>,
    cb: impl AsyncBounds + FnMut(T) -> Result<(), <Self::Database as Database>::Error>,
  ) -> impl AsyncBounds + Future<Output = Result<(), <Self::Database as Database>::Error>>
  where
    T: FromRecords<'exec, Self::Database> + Table<'entity, Database = Self::Database>,
    T::Associations: SqlWriter<T::Database>,
    str: ValueIdent<<Self::Database as Database>::Record<'exec>>,
    u64: for<'value> Decode<'value, Self::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any TableParams<'entity, T>: AsyncBounds,
  {
    async move {
      tp.write_select(buffer_cmd, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))?;
      let records = self.fetch_many_with_stmt(buffer_cmd.as_str(), (), |_| Ok(())).await?;
      buffer_cmd.clear();
      collect_entities_tables(buffer_cmd, &records, tp, cb)?;
      Ok(())
    }
  }

  /// Similar to `read_all` but expects more fine grained parameters.
  fn read_all_with_params<'entity, 'exec, T>(
    &'exec mut self,
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    select_limit: SelectLimit,
    tp: &TableParams<'entity, T>,
    where_str: &str,
    cb: impl AsyncBounds + FnMut(T) -> Result<(), <Self::Database as Database>::Error>,
  ) -> impl AsyncBounds + Future<Output = Result<(), <Self::Database as Database>::Error>>
  where
    T: FromRecords<'exec, Self::Database> + Table<'entity, Database = Self::Database>,
    T::Associations: SqlWriter<T::Database>,
    str: ValueIdent<<Self::Database as Database>::Record<'exec>>,
    u64: for<'value> Decode<'value, Self::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any TableParams<'entity, T>: AsyncBounds,
  {
    async move {
      tp.write_select(buffer_cmd, order_by, select_limit, &mut |b| {
        b.push_str(where_str);
        Ok(())
      })?;
      let records = self.fetch_many_with_stmt(buffer_cmd.as_str(), (), |_| Ok(())).await?;
      buffer_cmd.clear();
      collect_entities_tables(buffer_cmd, &records, tp, cb)?;
      Ok(())
    }
  }

  /// Fetches a single entity identified by `id`.
  fn read_by_id<'entity, 'exec, T>(
    &'exec mut self,
    buffer_cmd: &mut String,
    id: <T::Fields as TableFields<T::Database>>::IdValue,
    tp: &TableParams<'entity, T>,
  ) -> impl AsyncBounds + Future<Output = Result<T, <Self::Database as Database>::Error>>
  where
    T: FromRecords<'exec, Self::Database> + Table<'entity, Database = Self::Database>,
    T::Associations: SqlWriter<T::Database>,
    <T::Fields as TableFields<T::Database>>::IdValue: AsyncBounds,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any TableParams<'entity, T>: AsyncBounds,
  {
    async move {
      tp.write_select(buffer_cmd, SelectOrderBy::Ascending, SelectLimit::All, &mut |b| {
        write_select_field(
          b,
          T::TABLE_NAME,
          T::TABLE_NAME_ALIAS,
          tp.table_suffix(),
          tp.fields().id().name(),
        )?;
        b.write_fmt(format_args!(" = {id}")).map_err(From::from)?;
        Ok(())
      })?;
      let record = self.fetch_with_stmt(buffer_cmd.as_str(), ()).await?;
      buffer_cmd.clear();
      Ok(
        T::from_records(
          buffer_cmd,
          &record,
          &<Self::Database as Database>::Records::default(),
          tp.table_suffix(),
        )?
        .1,
      )
    }
  }

  /// Updates an entity overwriting all its parameters.
  fn update_all<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    table: &'entity T,
    table_params: &mut TableParams<'entity, T>,
  ) -> impl Future<Output = Result<(), <T::Database as Database>::Error>>
  where
    T: Table<'entity, Database = Self::Database>,
    T::Associations: SqlWriter<T::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut TableParams<'entity, T>: AsyncBounds,
    for<'any> &'any T: AsyncBounds,
  {
    async move {
      table_params.update_all_table_fields(table);
      table_params.write_update(&mut AuxNodes::default(), buffer_cmd, self).await?;
      Ok(())
    }
  }
}

impl<T> Crud for T where T: Executor {}

/// Collects all entities composed by all different rows.
///
/// One entity can constructed by more than one row.
#[inline]
fn collect_entities_tables<'entity, 'exec, D, T>(
  buffer_cmd: &mut String,
  records: &D::Records<'exec>,
  tp: &TableParams<'entity, T>,
  mut cb: impl FnMut(T) -> Result<(), D::Error>,
) -> Result<(), D::Error>
where
  D: Database,
  T: FromRecords<'exec, D> + Table<'entity, Database = D>,
  str: ValueIdent<D::Record<'exec>>,
  u64: for<'value> Decode<'value, D>,
{
  let mut curr_record_idx: usize = 0;
  loop {
    if curr_record_idx >= records.len() {
      break;
    }
    let suffix = tp.table_suffix();
    let skip =
      seek_related_entities(buffer_cmd, curr_record_idx, records, suffix, suffix, |entitiy| {
        cb(entitiy)
      })?;
    curr_record_idx = curr_record_idx.wrapping_add(skip);
  }
  Ok(())
}
