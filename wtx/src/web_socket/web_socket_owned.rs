use crate::{
  collections::{VectorU8, VectorUsize},
  misc::LeaseMut,
  net::{BufStreamReader, ConnectionState, StreamReader, StreamWriter},
  rng::Xorshift64,
  tls::{TlsCtx, TlsStreamReader, TlsStreamWriter},
  web_socket::{
    Frame, FrameMut, WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame,
    read_frame::{read_frame, read_frames},
    read_frame_info::ReadFrameInfo,
    web_socket_bridge::{WebSocketBridge, WebSocketBridgeData},
    web_socket_compression::{WebSocketCompression, WebSocketDecompression},
    write_frame::{write_frame, write_frames},
  },
};
use core::{marker::PhantomData, range::Range};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketReaderOwned<D, SR, TCX, const IS_CLIENT: bool> {
  pub(crate) is_in_continuation_frame: Option<IsInContinuationFrame>,
  pub(crate) max_payload_len: usize,
  pub(crate) nc: D,
  pub(crate) nc_rsv1: u8,
  pub(crate) network_buffer: BufStreamReader,
  pub(crate) no_masking: bool,
  pub(crate) phantom: PhantomData<SR>,
  pub(crate) reader_batches: VectorU8<(ReadFrameInfo, Range<usize>)>,
  pub(crate) reader_buffer: VectorUsize<u8>,
  pub(crate) rng: Xorshift64,
  pub(crate) stream_bridge: WebSocketBridge<IS_CLIENT>,
  pub(crate) stream_reader: TlsStreamReader<SR, TCX, IS_CLIENT>,
}

impl<D, SR, TCX, const IS_CLIENT: bool> WebSocketReaderOwned<D, SR, TCX, IS_CLIENT>
where
  D: WebSocketDecompression,
  SR: StreamReader,
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
    payload_origin: WebSocketPayloadOrigin,
  ) -> crate::Result<FrameMut<'frame>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    read_frame::<_, _, _, _, _, false, IS_CLIENT>(
      &mut self.is_in_continuation_frame,
      self.max_payload_len,
      &mut self.nc,
      self.nc_rsv1,
      &mut self.network_buffer,
      self.no_masking,
      payload_origin,
      &mut self.reader_buffer,
      &mut self.rng,
      &mut (&mut self.stream_reader, &mut ()),
      &self.stream_bridge,
      buffer,
      |el| {
        let _prev = ConnectionState::atomic_update(
          &el.0.common().connection_state,
          ConnectionState::on_remote_close,
        );
      },
      |local_stream| local_stream.0,
      |local_stream| local_stream.1,
    )
    .await
  }

  /// Reads all frames that are already available in the internal network buffer.
  #[inline]
  pub async fn read_frames<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut VectorUsize<u8>,
    payload_origin: WebSocketPayloadOrigin,
  ) -> crate::Result<impl Iterator<Item = FrameMut<'frame>>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    read_frames::<_, _, _, _, _, false, IS_CLIENT>(
      &mut self.is_in_continuation_frame,
      self.max_payload_len,
      &mut self.nc,
      self.nc_rsv1,
      &mut self.network_buffer,
      self.no_masking,
      payload_origin,
      &mut self.reader_batches,
      &mut self.reader_buffer,
      &mut self.rng,
      &mut (&mut self.stream_reader, &mut ()),
      &self.stream_bridge,
      buffer,
      |el| {
        let _prev = ConnectionState::atomic_update(
          &el.0.common().connection_state,
          ConnectionState::on_remote_close,
        );
      },
      |local_stream| local_stream.0,
      |local_stream| local_stream.1,
    )
    .await
  }
}

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketWriterOwned<C, SW, TCX, const IS_CLIENT: bool> {
  pub(crate) nc: C,
  pub(crate) nc_rsv1: u8,
  pub(crate) no_masking: bool,
  pub(crate) rng: Xorshift64,
  pub(crate) stream_writer: TlsStreamWriter<SW, TCX, IS_CLIENT>,
  pub(crate) writer_buffer: VectorUsize<u8>,
}

impl<C, SW, TCX, const IS_CLIENT: bool> WebSocketWriterOwned<C, SW, TCX, IS_CLIENT>
where
  C: WebSocketCompression,
  SW: StreamWriter,
  TCX: TlsCtx,
{
  /// Closes itself as well as the reader part
  #[inline]
  pub fn close_abruptly(&self) {
    self.stream_writer.close_abruptly();
  }

  /// Writes the reply frame returned by [`WebSocketBridge::listen`]. Returns `true` if the
  /// connection has been closed.
  #[inline]
  pub async fn manage_bridge_data(&mut self, data: WebSocketBridgeData) -> crate::Result<bool> {
    let should_stop = match (data.tls, data.ws) {
      (None, None) => true,
      (None, Some(mut ws)) => {
        self.write_frame(&mut ws).await?;
        ws.op_code().is_close()
      }
      (Some(tls), None) => self.stream_writer.manage_bridge_data(tls).await?,
      (Some(tls), Some(mut ws)) => {
        self.write_frame(&mut ws).await?;
        let should_stop_ws = ws.op_code().is_close();
        let should_stop_tls = self.stream_writer.manage_bridge_data(tls).await?;
        should_stop_tls || should_stop_ws
      }
    };
    Ok(should_stop)
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    write_frame::<_, _, _, _, IS_CLIENT>(
      frame,
      self.no_masking,
      &mut self.nc,
      self.nc_rsv1,
      &mut self.rng,
      &mut self.stream_writer,
      self.writer_buffer.lease_mut(),
      |el| {
        let _prev = ConnectionState::atomic_update(
          &el.common().connection_state,
          ConnectionState::on_local_close,
        );
        el.common().reader_waker.wake();
      },
    )
    .await
  }

  /// Writes multiple frames to the stream.
  #[inline]
  pub async fn write_frames<'frame, P>(
    &mut self,
    frames: impl IntoIterator<Item = &'frame mut Frame<P>>,
  ) -> crate::Result<()>
  where
    P: LeaseMut<[u8]> + 'frame,
  {
    write_frames::<_, _, _, _, IS_CLIENT>(
      frames,
      self.nc_rsv1,
      &mut self.nc,
      self.no_masking,
      &mut self.rng,
      &mut self.stream_writer,
      self.writer_buffer.lease_mut(),
      |el| {
        let _prev = ConnectionState::atomic_update(
          &el.common().connection_state,
          ConnectionState::on_local_close,
        );
        el.common().reader_waker.wake();
      },
    )
    .await
  }
}
