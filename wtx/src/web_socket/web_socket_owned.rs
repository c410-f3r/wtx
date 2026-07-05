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
use core::marker::PhantomData;

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
      |el| el.0.connection_state = ConnectionState::ReadClosed,
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
  /// Writes the reply frame returned by [`WebSocketBridge::listen`]. Returns `true` if the
  /// connection has been closed.
  #[inline]
  pub async fn manage_brige_data(&mut self, data: WebSocketBridgeData) -> crate::Result<()> {
    match (data.tls, data.ws) {
      (None, None) => {}
      (None, Some(mut ws)) => {
        self.do_write_frame::<_, true>(&mut ws).await?;
      }
      (Some(tls), None) => {
        self.stream_writer.manage_bridge_data(tls).await?;
      }
      (Some(tls), Some(mut ws)) => {
        self.stream_writer.manage_bridge_data(tls).await?;
        self.do_write_frame::<_, true>(&mut ws).await?;
      }
    }
    Ok(())
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
        el.connection_state =
          if IS_CLOSED { ConnectionState::Closed } else { ConnectionState::WriteClosed };
      },
    )
    .await
  }
}
