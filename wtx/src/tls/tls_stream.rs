use crate::{
  stream::{StreamReader, StreamWriter},
  tls::state::State,
};

/// Transport Layer Security (TLS)
///
/// This structure assumes a previously established handshake.
#[derive(Debug)]
pub struct TlsStream<S, TB, TM, const IS_CLIENT: bool> {
  pub(crate) state: State,
  pub(crate) stream: S,
  pub(crate) tb: TB,
  pub(crate) tm: TM,
}

impl<S, TB, TM, const IS_CLIENT: bool> TlsStream<S, TB, TM, IS_CLIENT> {
  /// Creates a new instance with a stream that supposedly already performed a handshake.
  #[inline]
  pub fn new(stream: S, tb: TB, tm: TM) -> Self {
    Self { state: State::Streaming, stream, tb, tm }
  }

  #[inline]
  pub fn into_parts(self) {}
}

impl<S, TB, TM, const IS_CLIENT: bool> StreamReader for TlsStream<S, TB, TM, IS_CLIENT>
where
  S: StreamReader,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    self.stream.read(bytes).await
  }
}

impl<S, TB, TM, const IS_CLIENT: bool> StreamWriter for TlsStream<S, TB, TM, IS_CLIENT>
where
  S: StreamWriter,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    self.stream.write_all(bytes).await
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    self.stream.write_all_vectored(bytes).await
  }
}
