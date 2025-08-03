/// The payload of a received frame can represent two things:
///
/// 1. A single uncompressed data that is stored in internal structures.
/// 2. One or more compressed or uncompressed concatenations that are stored in the provided buffer.
///
/// This distinction exists to ensure that there are as few copies as possible. If such a thing
/// isn't relevant, then you should directly access the payload bytes of the frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketReadFrameTy {
  /// Frame payload is located inside internal structures that are private.
  Internal,
  /// Frame payload is located in the provided buffer.
  Provided,
}
