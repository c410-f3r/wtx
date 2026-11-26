/// Collections error
#[derive(Clone, Copy, Debug)]
pub enum CollectionsError {
  /// It is not possible to add an element into an `Option` because it is already occupied.
  InsufficientOptionCapacity,
  /// An index that cuts an UTF-8 string makes the sequence invalid.
  InvalidUTF8Bound,
}
