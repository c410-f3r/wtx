/// The possible states of a stream operation that is receiving data.
#[derive(Clone, Copy, Debug)]
pub enum Http2RecvStatus<E, O> {
  /// Connection was closed, either locally or externally.
  ClosedConnection,
  /// Stream was closed, either locally or externally.
  ClosedStream,
  /// Remote peer sent an end of stream flag
  Eos(E),
  /// Signals an ongoing operation of an open stream
  Ongoing(O),
}

impl<E, O> Http2RecvStatus<E, O> {
  /// Is closed connection or stream
  #[inline]
  pub fn is_closed(&self) -> bool {
    matches!(self, Self::ClosedConnection | Self::ClosedStream)
  }
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
  pub fn is_closed(&self) -> bool {
    matches!(self, Self::ClosedConnection | Self::ClosedStream)
  }
}
