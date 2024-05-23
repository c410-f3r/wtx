use crate::database::orm::FullTableAssociation;

/// Groups tuples that form all associations of a table
pub trait TableAssociations {
  /// Yields all table associations
  fn full_associations(&self) -> impl Iterator<Item = FullTableAssociation>;
}

impl TableAssociations for () {
  #[inline]
  fn full_associations(&self) -> impl Iterator<Item = FullTableAssociation> {
    [].into_iter()
  }
}
