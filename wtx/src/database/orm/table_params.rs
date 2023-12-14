use crate::database::{
  orm::{FxHasher, Table, TableField},
  TableSuffix,
};
use core::{
  hash::{Hash, Hasher},
  marker::PhantomData,
};

/// A wrapper of instance values build based on [Table].
#[derive(Debug, PartialEq)]
pub struct TableParams<'entity, T>
where
  T: Table<'entity>,
{
  associations: T::Associations,
  fields: T::Fields,
  id_field: TableField<T::PrimaryKeyValue>,
  phantom: PhantomData<T>,
  ts: TableSuffix,
}

impl<'entity, T> TableParams<'entity, T>
where
  T: Table<'entity>,
{
  /// A new instance with all related table definition values created automatically.
  #[inline]
  pub fn new(ts: TableSuffix) -> Self {
    let (associations, fields) = T::type_instances(ts);
    Self {
      associations,
      fields,
      id_field: TableField::new(T::PRIMARY_KEY_NAME),
      phantom: PhantomData,
      ts,
    }
  }

  /// Table instance associations
  #[inline]
  pub fn associations(&self) -> &T::Associations {
    &self.associations
  }

  /// Mutable version of [associations]
  #[inline]
  pub fn associations_mut(&mut self) -> &mut T::Associations {
    &mut self.associations
  }

  /// Table instance fields
  #[inline]
  pub fn fields(&self) -> &T::Fields {
    &self.fields
  }

  /// Mutable version of [fields]
  #[inline]
  pub fn fields_mut(&mut self) -> &mut T::Fields {
    &mut self.fields
  }

  /// Field information related to the entity ID
  #[inline]
  pub fn id_field(&self) -> &TableField<T::PrimaryKeyValue> {
    &self.id_field
  }

  /// Mutable version of [id_field]
  #[inline]
  pub fn id_field_mut(&mut self) -> &mut TableField<T::PrimaryKeyValue> {
    &mut self.id_field
  }

  /// Used to write internal SQL operations
  #[inline]
  pub fn table_suffix(&self) -> TableSuffix {
    self.ts
  }

  /// Shortcut for `<T as Table<'_>>::update_all_table_fields(&entity, &mut table)`
  #[inline]
  pub fn update_all_table_fields(&mut self, entity: &'entity T) {
    T::update_all_table_fields(entity, self)
  }

  #[inline]
  pub(crate) fn instance_hash(&self) -> u64 {
    let mut fx_hasher = FxHasher::default();
    T::PRIMARY_KEY_NAME.hash(&mut fx_hasher);
    T::TABLE_NAME.hash(&mut fx_hasher);
    T::TABLE_NAME_ALIAS.hash(&mut fx_hasher);
    self.id_field().value().hash(&mut fx_hasher);
    fx_hasher.finish()
  }
}

impl<'entity, T> Default for TableParams<'entity, T>
where
  T: Table<'entity>,
{
  #[inline]
  fn default() -> Self {
    Self::new(0)
  }
}
