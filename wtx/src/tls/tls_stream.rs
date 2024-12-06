#[derive(Debug)]
pub struct TlsStream<A, S, const IS_CLIENT: bool> {
  aux: A,
  stream: S,
}

impl<A, S, const IS_CLIENT: bool> TlsStream<A, S, IS_CLIENT> {
  /// Creates a new instance with a stream that supposedly already performed a handshake.
  #[inline]
  pub fn new(aux: A, stream: S) -> Self {
    Self { aux, stream }
  }
}
