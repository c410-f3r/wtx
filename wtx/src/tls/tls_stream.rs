use crate::{
  stream::{StreamReader, StreamWriter},
  tls::{TlsConfig, state::State},
};

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
    Self { state: State::Stream, stream, tb, tm }
  }
}

impl<S, TB, TM> TlsStream<S, TB, TM, true> {
  #[inline]
  pub async fn connect(_: &TlsConfig<'_>, stream: S, tb: TB, tm: TM) -> crate::Result<Self> {
    Ok(Self { state: State::Stream, stream, tb, tm })
  }
}

impl<S, TB, TM, const IS_CLINET: bool> StreamReader for TlsStream<S, TB, TM, IS_CLINET>
where
  S: StreamReader,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    self.stream.read(bytes).await
  }
}

impl<S, TB, TM, const IS_CLINET: bool> StreamWriter for TlsStream<S, TB, TM, IS_CLINET>
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
