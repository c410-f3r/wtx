use crate::{
  collections::{SingleTypeStorage, VectorU8, VectorUsize},
  misc::LeaseMut,
  net::{BufStreamReader, ConnectionState, Stream},
  rng::Xorshift64,
  tls::{TlsCtx, TlsStream, TlsStreamBridge},
  web_socket::{
    Frame, FrameMut, WebSocketBridge, WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame,
    read_frame::{read_frame, read_frames},
    read_frame_info::ReadFrameInfo,
    web_socket_compression::NegotiatedWsCompression,
    write_frame::{write_frame, write_frames},
  },
};
use core::{marker::PhantomData, range::Range};

/// Auxiliary common structure used by [`WebSocketReaderMut`] and [`WebSocketWriterMut`]
#[derive(Debug)]
pub struct WebSocketCommonMut<'instance, NC, S, TCX, const IS_CLIENT: bool> {
  pub(crate) nc: &'instance mut NC,
  pub(crate) nc_rsv1: u8,
  pub(crate) rng: &'instance mut Xorshift64,
  pub(crate) stream: &'instance mut TlsStream<S, TCX, IS_CLIENT>,
}

impl<NC, S, TCX, const IS_CLIENT: bool> WebSocketCommonMut<'_, NC, S, TCX, IS_CLIENT> {
  /// See [`ConnectionState`].
  #[inline]
  pub const fn connection_state(&self) -> ConnectionState {
    self.stream.connection_state
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketReaderMut<'instance, NC, S, TCX, const IS_CLIENT: bool> {
  pub(crate) is_in_continuation_frame: &'instance mut Option<IsInContinuationFrame>,
  pub(crate) max_payload_len: usize,
  pub(crate) network_buffer: &'instance mut BufStreamReader,
  pub(crate) no_masking: bool,
  pub(crate) phantom: PhantomData<(NC, S, TCX)>,
  pub(crate) reader_batches: &'instance mut VectorU8<(ReadFrameInfo, Range<usize>)>,
  pub(crate) reader_buffer: &'instance mut VectorUsize<u8>,
}

impl<'instance, NC, S, TCX, const IS_CLIENT: bool>
  WebSocketReaderMut<'instance, NC, S, TCX, IS_CLIENT>
where
  NC: NegotiatedWsCompression,
  S: Stream,
  TCX: TlsCtx,
{
  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut VectorUsize<u8>,
    common: &mut WebSocketCommonMut<'instance, NC, S, TCX, IS_CLIENT>,
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
      |el| el.connection_state = el.connection_state.on_both_close(),
      |local_stream| local_stream,
      |local_stream| local_stream,
    )
    .await
  }

  /// Reads all frames that are already available in the internal network buffer.
  #[inline]
  pub async fn read_frames<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut VectorUsize<u8>,
    common: &mut WebSocketCommonMut<'instance, NC, S, TCX, IS_CLIENT>,
  ) -> crate::Result<impl Iterator<Item = FrameMut<'frame>> + use<'frame, NC, S, TCX, IS_CLIENT>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    read_frames::<_, _, _, _, _, _, _, _, true, IS_CLIENT>(
      self.is_in_continuation_frame,
      self.max_payload_len,
      common.nc,
      common.nc_rsv1,
      self.network_buffer.lease_mut(),
      self.no_masking,
      self.reader_batches.lease_mut(),
      self.reader_buffer.lease_mut(),
      common.rng,
      common.stream,
      &WebSocketBridge::new(TlsStreamBridge::new()),
      buffer,
      |el| el.connection_state = el.connection_state.on_both_close(),
      |local_stream| local_stream,
      |local_stream| local_stream,
    )
    .await
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketWriterMut<'instance, NC, S, TCX, const IS_CLIENT: bool> {
  pub(crate) no_masking: bool,
  pub(crate) phantom: PhantomData<(NC, S, TCX)>,
  pub(crate) writer_buffer: &'instance mut VectorUsize<u8>,
}

impl<'instance, NC, S, TCX, const IS_CLIENT: bool>
  WebSocketWriterMut<'instance, NC, S, TCX, IS_CLIENT>
where
  NC: NegotiatedWsCompression,
  S: Stream,
  TCX: TlsCtx,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(
    &mut self,
    common: &mut WebSocketCommonMut<'instance, NC, S, TCX, IS_CLIENT>,
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
      |el| el.connection_state = el.connection_state.on_local_close(),
    )
    .await
  }

  /// Writes multiple frames to the stream.
  #[inline]
  pub async fn write_frames<F, P>(
    &mut self,
    common: &mut WebSocketCommonMut<'instance, NC, S, TCX, IS_CLIENT>,
    frames: impl IntoIterator<Item = F>,
  ) -> crate::Result<()>
  where
    F: LeaseMut<Frame<P>> + SingleTypeStorage<Item = P>,
    P: LeaseMut<[u8]>,
  {
    write_frames::<_, _, _, _, _, IS_CLIENT>(
      frames,
      common.nc_rsv1,
      common.nc,
      self.no_masking,
      common.rng,
      common.stream,
      self.writer_buffer,
      |el| el.connection_state = el.connection_state.on_local_close(),
    )
    .await
  }
}
