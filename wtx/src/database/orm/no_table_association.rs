use crate::database::orm::{
  AuxNodes, FullTableAssociation, SelectLimit, SelectOrderBy, SqlWriter, TableAssociations,
  TableSourceAssociation,
};
use alloc::string::String;
use core::{array, marker::PhantomData};

/// For entities that don't have associations
#[derive(Debug)]
pub struct NoTableAssociation<E>(PhantomData<E>);

impl<E> NoTableAssociation<E> {
  /// Creates a new instance regardless of `E`
  #[inline]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<E> TableAssociations for NoTableAssociation<E> {
  type FullTableAssociations = array::IntoIter<FullTableAssociation, 0>;

  #[inline]
  fn full_associations(&self) -> Self::FullTableAssociations {
    [].into_iter()
  }
}

impl<E> SqlWriter for NoTableAssociation<E>
where
  E: From<crate::Error>,
{
  type Error = E;

  #[inline]
  fn write_delete(&self, _: &mut AuxNodes, _: &mut String) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn write_insert<V>(
    &self,
    _: &mut AuxNodes,
    _: &mut String,
    _: &mut Option<TableSourceAssociation<'_, V>>,
  ) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn write_select(
    &self,
    _: &mut String,
    _: SelectOrderBy,
    _: SelectLimit,
    _: &mut impl FnMut(&mut String) -> Result<(), Self::Error>,
  ) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn write_select_associations(&self, _: &mut String) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn write_select_fields(&self, _: &mut String) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn write_select_orders_by(&self, _: &mut String) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn write_update(&self, _: &mut AuxNodes, _: &mut String) -> Result<(), Self::Error> {
    Ok(())
  }
}
