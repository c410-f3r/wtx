use crate::{
  collections::Vector,
  misc::LeaseMut,
  rng::Xorshift64,
  stream::{BufStreamReader, StreamReader, StreamWriter},
  sync::{Arc, AtomicBool},
  tls::{TlsMode, TlsStreamReader, TlsStreamWriter},
  web_socket::{
    Frame, FrameMut, WebSocketPayloadOrigin,
    is_in_continuation_frame::IsInContinuationFrame,
    web_socket_bridge::{WebSocketBridge, WebSocketBridgeData},
    web_socket_compression::{WebSocketCompression, WebSocketDecompression},
    web_socket_parts::web_socket_generic::{WebSocketReaderGeneric, WebSocketWriterGeneric},
  },
};
use core::{marker::PhantomData, sync::atomic::Ordering};

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketReaderOwned<D, SR, TM, const IS_CLIENT: bool> {
  pub(crate) connection_state: Arc<AtomicBool>,
  pub(crate) is_in_continuation_frame: Option<IsInContinuationFrame>,
  pub(crate) nc: D,
  pub(crate) nc_rsv1: u8,
  pub(crate) phantom: PhantomData<SR>,
  pub(crate) reader_part: WebSocketReaderGeneric<BufStreamReader, Vector<u8>, IS_CLIENT>,
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
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    let rslt = self
      .reader_part
      .read_frame_owned(
        &mut connection_state,
        &mut self.is_in_continuation_frame,
        &mut self.nc,
        self.nc_rsv1,
        payload_origin,
        &mut self.rng,
        &self.stream_bridge,
        &mut self.stream_reader,
        buffer,
      )
      .await?;
    self.connection_state.store(connection_state.into(), Ordering::Relaxed);
    Ok(rslt)
  }
}

impl<NC, SR, TM, const IS_CLIENT: bool> Drop for WebSocketReaderOwned<NC, SR, TM, IS_CLIENT> {
  #[inline]
  fn drop(&mut self) {
    let _rslt = self.stream_bridge.data().update(|elem| (true, elem.1));
    self.stream_bridge.waker().wake();
  }
}

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketWriterOwned<C, SW, TM, const IS_CLIENT: bool> {
  pub(crate) connection_state: Arc<AtomicBool>,
  pub(crate) nc: C,
  pub(crate) nc_rsv1: u8,
  pub(crate) rng: Xorshift64,
  pub(crate) stream_writer: TlsStreamWriter<SW, TM, IS_CLIENT>,
  pub(crate) writer_part: WebSocketWriterGeneric<Vector<u8>, IS_CLIENT>,
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
        self.write_frame(&mut ws).await?;
      }
      (Some(tls), None) => {
        self.stream_writer.manage_bridge_data(tls).await?;
      }
      (Some(tls), Some(mut ws)) => {
        self.stream_writer.manage_bridge_data(tls).await?;
        self.write_frame(&mut ws).await?;
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
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    self
      .writer_part
      .write_frame(
        &mut connection_state,
        frame,
        &mut self.nc,
        self.nc_rsv1,
        &mut self.rng,
        &mut self.stream_writer,
      )
      .await?;
    self.connection_state.store(connection_state.into(), Ordering::Relaxed);
    Ok(())
  }
}
