/// Net error
#[derive(Debug, Clone, Copy)]
pub enum NetError {
  /// Connections should gracefully stop but the peer unexpectedly closed by stream.
  AbruptDisconnect,
  /// External actor sent a payload greater than the maximum capacity
  CapacityOverflow,
  /// The instance is configured to prevent the removal of contents
  ForbiddenClear,
  /// Nothing can resolve a domain into sockets.
  NoResolutionBackend,
  /// Unexpected end of file when reading from a stream.
  UnexpectedStreamReadEOF,
  /// Unexpected end of file when writing to a stream.
  UnexpectedStreamWriteEOF,
  /// It is not possible to write more than 8 slices at once.
  VectoredWriteOverflow,
}
