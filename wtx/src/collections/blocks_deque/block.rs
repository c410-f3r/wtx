use core::ops::Range;

/// A block that composes `BlocksDeque`.
#[derive(Debug, PartialEq)]
pub struct Block<D, M> {
  /// Opaque data
  pub data: D,
  /// Miscellaneous
  pub misc: M,
  /// Range
  pub range: Range<usize>,
}
