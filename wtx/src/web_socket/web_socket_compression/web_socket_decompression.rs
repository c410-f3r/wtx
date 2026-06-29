use crate::codec::Decompression;

/// WebSocket Decompression
pub trait WebSocketDecompression: Decompression {
  /// No Content Takeover
  fn no_context_takeover(&self) -> bool;
}

impl WebSocketDecompression for () {
  #[inline]
  fn no_context_takeover(&self) -> bool {
    false
  }
}

impl<T> WebSocketDecompression for Option<T>
where
  T: WebSocketDecompression,
{
  #[inline]
  fn no_context_takeover(&self) -> bool {
    if let Some(elem) = self { elem.no_context_takeover() } else { false }
  }
}
