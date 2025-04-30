use core::ops::Range;

/// Block
#[derive(Debug, PartialEq)]
pub struct Block<D, M> {
  /// Opaque data
  pub data: D,
  /// Miscellaneous
  pub misc: M,
  /// Range
  pub range: Range<usize>,
}
