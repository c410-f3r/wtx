/// PostgreSQL error
#[derive(Debug)]
pub enum PostgresError {
  /// Not-A-Number is not supported
  DecimalCanNotBeConvertedFromNaN,
  /// There are no sufficient bytes to decoding an element
  DecodingError,
  /// There are no bytes left to build a `DbError`
  InsufficientDbErrorBytes,
  /// Invalid IP format
  InvalidIpFormat,
  /// JSONB is the only supported JSON format
  InvalidJsonFormat,
  /// Postgres does not support large unsigned integers. For example, `u8` can only be stored
  /// and read with numbers up to 127.
  InvalidPostgresUint,
  /// Received bytes don't compose a valid record.
  InvalidPostgresRecord,
  /// The iterator that composed a `RecordValues` does not contain a corresponding length.
  InvalidRecordValuesIterator,
  /// It is required to connect using a TLS channel but the server didn't provide any. Probably
  /// because the connection is unencrypted.
  MissingChannel,
  /// Expected one record but got none.
  NoRecord,
  /// It is required to connect without using a TLS channel but the server only provided a way to
  /// connect using channels. Probably because the connection is encrypted.
  RequiredChannel,
  /// Server does not support encryption
  ServerDoesNotSupportEncryption,
  /// A query
  StatementHashCollision,
  /// Received size differs from expected size.
  UnexpectedBufferSize {
    /// Expected
    expected: u32,
    /// Received
    received: u32,
  },
  /// Received an unexpected message type.
  UnexpectedDatabaseMessage {
    /// Received
    received: u8,
  },
  /// Received an expected message type but the related bytes are in an unexpected state.
  UnexpectedDatabaseMessageBytes,
  /// Bytes don't represent expected type
  UnexpectedValueFromBytes {
    /// Expected
    expected: &'static str,
  },
  /// The system does not support a requested authentication method.
  UnknownAuthenticationMethod,
  /// The system does not support a provided parameter.
  UnknownConfigurationParameter,
  /// Received a statement ID that is not present in the local cache.
  UnknownStatementId,
  /// The system only supports decimals with 64 digits.
  VeryLargeDecimal,
}
