use crate::collection::ShortStrU8;

/// Database Error
#[derive(Debug, PartialEq)]
pub enum DatabaseError {
  /// Query returned more than one record
  ExpectedAtMostOneRecord,
  /// The method `expand` of `StatementBuilder` must be called only once.
  InconsistentStatementBuilder,
  /// A "null" field received from the database was decoded as a non-nullable type or value.
  MissingFieldDataInDecoding(ShortStrU8<'static>, Option<u16>),
  /// Expected one record but got none.
  MissingSingleRecord,
  /// Received size differs from expected size.
  UnexpectedBufferSize {
    /// Expected
    expected: u32,
    /// Received
    received: u32,
  },
  /// Expected no records but got at least one.
  UnexpectedRecords,
  /// Bytes don't represent expected type
  UnexpectedValueFromBytes {
    /// Expected
    expected: ShortStrU8<'static>,
  },
  /// Received a statement ID that is not present in the local cache.
  UnknownStatementId,
}
