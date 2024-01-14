use crate::{
  database::{
    executor::Executor,
    orm::{
      seek_related_entities, write_select_field, InitialInsertValue, SelectLimit, SelectOrderBy,
      SqlWriter, Table, TableParams,
    },
    Database, Decode, FromRecords, Records, ValueIdent,
  },
  misc::AsyncBounds,
};
use alloc::{string::String, vec::Vec};
use core::{fmt::Write, future::Future};

/// Create, read, update and delete entities.
pub trait Crud: Executor {
  /// Inserts a new table record represented by `table_params`.
  fn create<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    table: &'entity T,
    table_params: &mut TableParams<'entity, T>,
  ) -> impl AsyncBounds + Future<Output = Result<(), T::Error>>
  where
    T: Table<'entity, Error = <Self::Database as Database>::Error>,
    T::Associations: SqlWriter<Error = T::Error>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut TableParams<'entity, T>: AsyncBounds,
    for<'any> &'any T: AsyncBounds,
  {
    async move {
      table_params.update_all_table_fields(table);
      table_params.write_insert::<InitialInsertValue>(
        &mut <_>::default(),
        buffer_cmd,
        &mut None,
      )?;
      let _ = self.execute_with_stmt(buffer_cmd.as_str(), ()).await?;
      Ok(())
    }
  }

  /// Deletes all rows that compose a table.
  fn delete_all<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    table: &'entity T,
    table_params: &mut TableParams<'entity, T>,
  ) -> impl AsyncBounds + Future<Output = Result<(), T::Error>>
  where
    T: Table<'entity, Error = <Self::Database as Database>::Error>,
    T::Associations: SqlWriter<Error = T::Error>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut TableParams<'entity, T>: AsyncBounds,
    for<'any> &'any T: AsyncBounds,
  {
    async move {
      table_params.update_all_table_fields(table);
      table_params.write_delete(&mut <_>::default(), buffer_cmd)?;
      let _ = self.execute_with_stmt(buffer_cmd.as_str(), ()).await?;
      Ok(())
    }
  }

  /// Fetches all entities from the database.
  fn read_all<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    results: &mut Vec<T>,
    tp: &TableParams<'entity, T>,
  ) -> impl AsyncBounds + Future<Output = Result<(), <Self::Database as Database>::Error>>
  where
    T: FromRecords<Self::Database> + Table<'entity, Error = <Self::Database as Database>::Error>,
    T::Associations: SqlWriter<Error = <Self::Database as Database>::Error>,
    str: for<'rec> ValueIdent<<Self::Database as Database>::Record<'rec>>,
    u64: for<'value> Decode<'value, Self::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut Vec<T>: AsyncBounds,
    for<'any> &'any TableParams<'entity, T>: AsyncBounds,
  {
    async move {
      tp.write_select(buffer_cmd, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))?;
      let records = self.fetch_many_with_stmt(buffer_cmd.as_str(), (), |_| Ok(())).await?;
      buffer_cmd.clear();
      collect_entities_tables(buffer_cmd, &records, results, tp)?;
      Ok(())
    }
  }

  /// Similar to `read_all` but expects more fine grained parameters.
  fn read_all_with_params<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    order_by: SelectOrderBy,
    results: &mut Vec<T>,
    select_limit: SelectLimit,
    tp: &TableParams<'entity, T>,
    where_str: &str,
  ) -> impl AsyncBounds + Future<Output = Result<(), <Self::Database as Database>::Error>>
  where
    T: FromRecords<Self::Database> + Table<'entity, Error = <Self::Database as Database>::Error>,
    T::Associations: SqlWriter<Error = <Self::Database as Database>::Error>,
    str: for<'rec> ValueIdent<<Self::Database as Database>::Record<'rec>>,
    u64: for<'value> Decode<'value, Self::Database>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut Vec<T>: AsyncBounds,
    for<'any> &'any TableParams<'entity, T>: AsyncBounds,
  {
    async move {
      tp.write_select(buffer_cmd, order_by, select_limit, &mut |b| {
        b.push_str(where_str);
        Ok(())
      })?;
      let records = self.fetch_many_with_stmt(buffer_cmd.as_str(), (), |_| Ok(())).await?;
      buffer_cmd.clear();
      collect_entities_tables(buffer_cmd, &records, results, tp)?;
      Ok(())
    }
  }

  /// Fetches a single entity identified by `id`.
  fn read_by_id<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    id: T::PrimaryKeyValue,
    tp: &TableParams<'entity, T>,
  ) -> impl AsyncBounds + Future<Output = Result<T, <Self::Database as Database>::Error>>
  where
    T: FromRecords<Self::Database> + Table<'entity, Error = <Self::Database as Database>::Error>,
    T::Associations: SqlWriter<Error = <Self::Database as Database>::Error>,
    T::PrimaryKeyValue: AsyncBounds,
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
          tp.id_field().name(),
        )?;
        b.write_fmt(format_args!(" = {id}")).map_err(From::from)?;
        Ok(())
      })?;
      let record = self.fetch_with_stmt(buffer_cmd.as_str(), ()).await?;
      buffer_cmd.clear();
      Ok(T::from_records(buffer_cmd, &record, &<_>::default(), tp.table_suffix())?.1)
    }
  }

  /// Updates an entity overwriting all its parameters.
  fn update_all<'entity, T>(
    &mut self,
    buffer_cmd: &mut String,
    table: &'entity T,
    table_params: &mut TableParams<'entity, T>,
  ) -> impl AsyncBounds + Future<Output = Result<(), T::Error>>
  where
    T: Table<'entity, Error = <Self::Database as Database>::Error>,
    T::Associations: SqlWriter<Error = T::Error>,
    for<'any> &'any mut Self: AsyncBounds,
    for<'any> &'any mut TableParams<'entity, T>: AsyncBounds,
    for<'any> &'any T: AsyncBounds,
  {
    async move {
      table_params.update_all_table_fields(table);
      table_params.write_update(&mut <_>::default(), buffer_cmd)?;
      let _ = self.execute_with_stmt(buffer_cmd.as_str(), ()).await?;
      Ok(())
    }
  }
}

impl<T> Crud for T where T: Executor {}

/// Collects all entities composed by all different rows.
///
/// One entity can constructed by more than one row.
#[inline]
fn collect_entities_tables<'entity, D, T>(
  buffer_cmd: &mut String,
  records: &D::Records<'_>,
  results: &mut Vec<T>,
  tp: &TableParams<'entity, T>,
) -> Result<(), <T as Table<'entity>>::Error>
where
  D: Database,
  T: FromRecords<D> + Table<'entity, Error = D::Error>,
  str: for<'rec> ValueIdent<D::Record<'rec>>,
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
        results.push(entitiy);
        Ok(())
      })?;
    curr_record_idx = curr_record_idx.wrapping_add(skip);
  }
  Ok(())
}
