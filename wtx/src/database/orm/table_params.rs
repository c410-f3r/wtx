use crate::{
  database::{
    orm::{table_fields::TableFields, Table},
    TableSuffix,
  },
  misc::FxHasher,
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
  phantom: PhantomData<T>,
  ts: TableSuffix,
}

impl<'entity, T> TableParams<'entity, T>
where
  T: Table<'entity>,
{
  /// A new instance with all related table definition values created automatically.
  #[inline]
  pub fn from_ts(ts: TableSuffix) -> Self {
    let (associations, fields) = T::type_instances(ts);
    Self { associations, fields, phantom: PhantomData, ts }
  }

  /// Table instance associations
  #[inline]
  pub const fn associations(&self) -> &T::Associations {
    &self.associations
  }

  /// Mutable version of [associations]
  #[inline]
  pub fn associations_mut(&mut self) -> &mut T::Associations {
    &mut self.associations
  }

  /// Table instance fields
  #[inline]
  pub const fn fields(&self) -> &T::Fields {
    &self.fields
  }

  /// Mutable version of [fields]
  #[inline]
  pub fn fields_mut(&mut self) -> &mut T::Fields {
    &mut self.fields
  }

  /// Used to write internal SQL operations
  #[inline]
  pub const fn table_suffix(&self) -> TableSuffix {
    self.ts
  }

  /// Shortcut for `<T as Table<'_>>::update_all_table_fields(&entity, &mut table)`
  #[inline]
  pub fn update_all_table_fields(&mut self, entity: &'entity T) {
    T::update_all_table_fields(entity, self)
  }

  pub(crate) fn instance_hash(&self) -> u64 {
    let mut fx_hasher = FxHasher::default();
    self.fields().id().name().hash(&mut fx_hasher);
    T::TABLE_NAME.hash(&mut fx_hasher);
    T::TABLE_NAME_ALIAS.hash(&mut fx_hasher);
    self.fields().id().value().hash(&mut fx_hasher);
    fx_hasher.finish()
  }
}

impl<'entity, T> Default for TableParams<'entity, T>
where
  T: Table<'entity>,
{
  #[inline]
  fn default() -> Self {
    Self::from_ts(0)
  }
}
