use crate::collections::{ShortBoxStrU8, ShortStrU8};

/// An enum for ods specific errors
#[derive(Debug)]
pub enum OdsError {
  /// Exceed cell limit
  ExceedCellLimit,
  /// Invalid ODS mime
  InvalidMime,
  /// Mismatched qanem
  MismatchedQname {
    /// Expected
    expected: ShortStrU8<'static>,
  },
  /// Files protected by passwords are unsupported
  ProtectedOds,
  /// Worksheet does not exist
  UnknownWorksheet(ShortBoxStrU8),
}
