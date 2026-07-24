use crate::codec::protocol::JsonRpcResponseError;

/// Decode/EnCode error
#[derive(Debug)]
pub enum CodecError {
  /// Quotes don't conform with `RFC-4180`
  CsvInvalidQuotes,
  /// CSV line is too large to parse
  CsvLineOverflow,
  /// Invalid PEM block
  InvalidPemBlock,
  /// Invalid PEM label
  InvalidPemLabelBegin,
  /// Mismatched PEM labels
  MismatchedPemLabel,
  /// JSON-RPC response error
  JsonRpcDecoderErr(JsonRpcResponseError),
  /// `wtx` can not perform this operation due to known limitations.
  UnsupportedOperation,
}
