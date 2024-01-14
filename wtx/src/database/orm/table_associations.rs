use crate::database::orm::FullTableAssociation;

/// Groups tuples that form all associations of a table
pub trait TableAssociations {
  /// See [TableAssociations::full_associations]
  type FullTableAssociations: Iterator<Item = FullTableAssociation>;

  /// Yields all table associations
  fn full_associations(&self) -> Self::FullTableAssociations;
}

impl TableAssociations for () {
  type FullTableAssociations = core::array::IntoIter<FullTableAssociation, 0>;

  #[inline]
  fn full_associations(&self) -> Self::FullTableAssociations {
    [].into_iter()
  }
}
