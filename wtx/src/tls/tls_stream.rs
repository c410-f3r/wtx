use crate::{
  stream::{StreamReader, StreamWriter},
  tls::{TlsStreamReader, TlsStreamWriter, state::State},
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
  pub fn into_split<SR, SW>(
    self,
    split: impl FnOnce(S) -> (SR, SW),
  ) -> (TlsStreamReader<SR>, TlsStreamWriter<SW>) {
    let (stream_reader, stream_writer) = split(self.stream);
    (TlsStreamReader { stream_reader }, TlsStreamWriter { stream_writer })
  }

  /// Channel binding data as defined in [RFC 5929].
  ///
  /// [RFC 5929]: https://tools.ietf.org/html/rfc5929
  #[inline]
  pub fn tls_server_end_point(&self) -> [u8; 0] {
    []
  }
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
