/// gRPC status codes.
#[derive(Clone, Copy, Debug)]
pub enum GrpcStatusCode {
  /// Not an error; returned on success.
  Ok = 0,
  /// The operation was cancelled, typically by the caller.
  Cancelled = 1,
  /// Unknown error.
  Unknown = 2,
  /// The client specified an invalid argument.
  InvalidArgument = 3,
  /// The deadline expired before the operation could complete.
  DeadlineExceeded = 4,
  /// Some requested entity (e.g., file or directory) was not found.
  NotFound = 5,
  /// The entity that a client attempted to create (e.g., file or directory) already exists.
  AlreadyExists = 6,
  /// The caller does not have permission to execute the specified operation.
  PermissionDenied = 7,
  /// Some resource has been exhausted, perhaps a per-user quota, or perhaps the entire file
  /// system is out of space.
  ResourceExhausted = 8,
  /// The operation was rejected because the system is not in a state required for the
  /// operation's execution.
  FailedPrecondition = 9,
  /// The operation was aborted, typically due to a concurrency issue such as a sequencer check
  /// failure or transaction abort.
  Aborted = 10,
  /// The operation was attempted past the valid range.
  OutOfRange = 11,
  /// The operation is not implemented or is not supported/enabled in this service.
  Unimplemented = 12,
  /// Internal error.
  Internal = 13,
  /// The service is currently unavailable.
  Unavailable = 14,
  /// Unrecoverable data loss or corruption.
  DataLoss = 15,
  /// The request does not have valid authentication credentials for the operation.
  Unauthenticated = 16,
}

impl GrpcStatusCode {
  /// String representation of the associated number
  #[inline]
  pub fn as_str(self) -> &'static str {
    match self {
      GrpcStatusCode::Ok => "0",
      GrpcStatusCode::Cancelled => "1",
      GrpcStatusCode::Unknown => "2",
      GrpcStatusCode::InvalidArgument => "3",
      GrpcStatusCode::DeadlineExceeded => "4",
      GrpcStatusCode::NotFound => "5",
      GrpcStatusCode::AlreadyExists => "6",
      GrpcStatusCode::PermissionDenied => "7",
      GrpcStatusCode::ResourceExhausted => "8",
      GrpcStatusCode::FailedPrecondition => "9",
      GrpcStatusCode::Aborted => "10",
      GrpcStatusCode::OutOfRange => "11",
      GrpcStatusCode::Unimplemented => "12",
      GrpcStatusCode::Internal => "13",
      GrpcStatusCode::Unavailable => "14",
      GrpcStatusCode::DataLoss => "15",
      GrpcStatusCode::Unauthenticated => "16",
    }
  }
}
