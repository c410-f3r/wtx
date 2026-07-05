use crate::{
  collections::Vector,
  misc::{ConnectionState, LeaseMut},
  rng::Xorshift64,
  stream::{BufStreamReader, Stream},
  tls::{TlsMode, TlsStream, TlsStreamBridge},
  web_socket::{
    Frame, FrameMut, WebSocketBridge, WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame, read_frame::read_frame,
    web_socket_compression::NegotiatedWsCompression, write_frame::write_frame,
  },
};
use core::marker::PhantomData;

/// Auxiliary common structure used by [`WebSocketReaderMut`] and [`WebSocketWriterMut`]
#[derive(Debug)]
pub struct WebSocketCommonMut<'instance, NC, S, TM, const IS_CLIENT: bool> {
  pub(crate) nc: &'instance mut NC,
  pub(crate) nc_rsv1: u8,
  pub(crate) rng: &'instance mut Xorshift64,
  pub(crate) stream: &'instance mut TlsStream<S, TM, IS_CLIENT>,
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketReaderMut<'instance, NC, S, TM, const IS_CLIENT: bool> {
  pub(crate) is_in_continuation_frame: &'instance mut Option<IsInContinuationFrame>,
  pub(crate) max_payload_len: usize,
  pub(crate) network_buffer: &'instance mut BufStreamReader,
  pub(crate) no_masking: bool,
  pub(crate) phantom: PhantomData<(NC, S, TM)>,
  pub(crate) reader_buffer: &'instance mut Vector<u8>,
}

impl<'instance, NC, S, TM, const IS_CLIENT: bool>
  WebSocketReaderMut<'instance, NC, S, TM, IS_CLIENT>
where
  NC: NegotiatedWsCompression,
  S: Stream,
  TM: TlsMode,
{
  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut Vector<u8>,
    common: &mut WebSocketCommonMut<'instance, NC, S, TM, IS_CLIENT>,
    payload_origin: WebSocketPayloadOrigin,
  ) -> crate::Result<FrameMut<'frame>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    read_frame::<_, _, _, _, _, true, IS_CLIENT>(
      self.is_in_continuation_frame,
      self.max_payload_len,
      common.nc,
      common.nc_rsv1,
      self.network_buffer.lease_mut(),
      self.no_masking,
      payload_origin,
      self.reader_buffer.lease_mut(),
      common.rng,
      common.stream,
      &WebSocketBridge::new(TlsStreamBridge::new()),
      buffer,
      |el| el.connection_state = ConnectionState::Closed,
      |local_stream| local_stream,
      |local_stream| local_stream,
    )
    .await
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketWriterMut<'instance, NC, S, TM, const IS_CLIENT: bool> {
  pub(crate) no_masking: bool,
  pub(crate) phantom: PhantomData<(NC, S, TM)>,
  pub(crate) writer_buffer: &'instance mut Vector<u8>,
}

impl<'instance, NC, S, TM, const IS_CLIENT: bool>
  WebSocketWriterMut<'instance, NC, S, TM, IS_CLIENT>
where
  NC: NegotiatedWsCompression,
  S: Stream,
  TM: TlsMode,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(
    &mut self,
    common: &mut WebSocketCommonMut<'instance, NC, S, TM, IS_CLIENT>,
    frame: &mut Frame<P>,
  ) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    write_frame::<_, _, _, _, IS_CLIENT>(
      frame,
      self.no_masking,
      common.nc,
      common.nc_rsv1,
      common.rng,
      common.stream,
      self.writer_buffer,
      |el| el.connection_state = ConnectionState::WriteClosed,
    )
    .await
  }
}
