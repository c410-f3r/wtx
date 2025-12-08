/// The possible states of a stream operation that is receiving data.
#[derive(Clone, Copy, Debug)]
pub enum Http2RecvStatus<EOS, ONG> {
  /// Connection was closed (abruptly or not), either locally or externally.
  ClosedConnection,
  /// Stream was closed (abruptly or not), either locally or externally.
  ClosedStream(EOS),
  /// Remote peer sent an end of stream flag, which indicates a successful stream.
  Eos(EOS),
  /// Signals an ongoing operation of an open stream
  ///
  /// Unreachable if an higher operation is called.
  Ongoing(ONG),
}

/// The possible states of a stream operation that is sending data.
#[derive(Clone, Copy, Debug)]
pub enum Http2SendStatus {
  /// Connection was closed, either locally or externally.
  ClosedConnection,
  /// Stream was closed, either locally or externally.
  ClosedStream,
  /// The stream is in a state where it is impossible to locally send data.
  InvalidState,
  /// Successful operation
  Ok,
}

impl Http2SendStatus {
  /// Is closed connection or stream
  #[inline]
  pub const fn is_closed(&self) -> bool {
    matches!(self, Self::ClosedConnection | Self::ClosedStream)
  }
}
