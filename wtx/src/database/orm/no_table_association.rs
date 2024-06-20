use crate::database::{
  orm::{AuxNodes, FullTableAssociation, SelectLimit, SelectOrderBy, SqlWriter, TableAssociations},
  Database, Executor,
};
use alloc::string::String;
use core::marker::PhantomData;

/// For entities that don't have associations
#[derive(Debug, Default)]
pub struct NoTableAssociation<E>(PhantomData<E>);

impl<E> NoTableAssociation<E> {
  /// Creates a new instance regardless of `E`
  #[inline]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<E> TableAssociations for NoTableAssociation<E> {
  #[inline]
  fn full_associations(&self) -> impl Iterator<Item = FullTableAssociation> {
    [].into_iter()
  }
}

impl<DB, E> SqlWriter<DB> for NoTableAssociation<E>
where
  DB: Database,
  E: From<crate::Error>,
{
  #[inline]
  async fn write_delete<EX>(
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
