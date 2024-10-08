/// FDSFSD
#[derive(Clone, Copy, Debug)]
pub enum Http2Status {
  /// Closed connection
  ClosedConnection,
  /// Closed stream
  ClosedStream,
  /// End Of Stream
  Eos,
  /// Successful operation
  Ok,
}
