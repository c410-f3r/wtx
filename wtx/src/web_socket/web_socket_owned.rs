use crate::{
  collections::Vector,
  misc::{ConnectionState, LeaseMut},
  rng::Xorshift64,
  stream::{BufStreamReader, StreamReader, StreamWriter},
  tls::{TlsMode, TlsStreamReader, TlsStreamWriter},
  web_socket::{
    Frame, FrameMut, WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame,
    read_frame::read_frame,
    web_socket_bridge::{WebSocketBridge, WebSocketBridgeData},
    web_socket_compression::{WebSocketCompression, WebSocketDecompression},
    write_frame::write_frame,
  },
};
use core::{marker::PhantomData, sync::atomic::Ordering};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketReaderOwned<D, SR, TM, const IS_CLIENT: bool> {
  pub(crate) is_in_continuation_frame: Option<IsInContinuationFrame>,
  pub(crate) max_payload_len: usize,
  pub(crate) nc: D,
  pub(crate) nc_rsv1: u8,
  pub(crate) network_buffer: BufStreamReader,
  pub(crate) no_masking: bool,
  pub(crate) phantom: PhantomData<SR>,
  pub(crate) reader_buffer: Vector<u8>,
  pub(crate) rng: Xorshift64,
  pub(crate) stream_bridge: WebSocketBridge<IS_CLIENT>,
  pub(crate) stream_reader: TlsStreamReader<SR, TM, IS_CLIENT>,
}

impl<D, SR, TM, const IS_CLIENT: bool> WebSocketReaderOwned<D, SR, TM, IS_CLIENT>
where
  D: WebSocketDecompression,
  SR: StreamReader,
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
      |el| el.0.connection_state_raw().store(ConnectionState::ReadClosed.into(), Ordering::Relaxed),
      |local_stream| local_stream.0,
      |local_stream| local_stream.1,
    )
    .await
  }
}

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketWriterOwned<C, SW, TM, const IS_CLIENT: bool> {
  pub(crate) nc: C,
  pub(crate) nc_rsv1: u8,
  pub(crate) no_masking: bool,
  pub(crate) rng: Xorshift64,
  pub(crate) stream_writer: TlsStreamWriter<SW, TM, IS_CLIENT>,
  pub(crate) writer_buffer: Vector<u8>,
}

impl<C, SW, TM, const IS_CLIENT: bool> WebSocketWriterOwned<C, SW, TM, IS_CLIENT>
where
  C: WebSocketCompression,
  SW: StreamWriter,
  TM: TlsMode,
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
        self.do_write_frame::<_, true>(&mut ws).await?;
        if ws.op_code().is_close() {
          self.close_abruptly();
          true
        } else {
          false
        }
      }
      (Some(tls), None) => self.stream_writer.manage_bridge_data(tls).await?,
      (Some(tls), Some(mut ws)) => {
        self.do_write_frame::<_, true>(&mut ws).await?;
        let should_stop_ws = ws.op_code().is_close();
        let should_stop_tls = self.stream_writer.manage_bridge_data(tls).await?;
        if should_stop_tls {
          true
        } else if should_stop_ws {
          self.close_abruptly();
          true
        } else {
          false
        }
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
    self.do_write_frame::<_, false>(frame).await
  }

  #[inline]
  async fn do_write_frame<P, const IS_CLOSED: bool>(
    &mut self,
    frame: &mut Frame<P>,
  ) -> crate::Result<()>
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
        let value =
          if IS_CLOSED { ConnectionState::ClosedGracefully } else { ConnectionState::WriteClosed };
        el.connection_state_raw().store(value.into(), Ordering::Relaxed);
      },
    )
    .await
  }
}
