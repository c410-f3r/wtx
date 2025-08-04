use crate::{
  collection::Vector,
  misc::{LeaseMut, net::PartitionedFilledBuffer},
  rng::Rng,
  stream::{StreamReader, StreamWriter},
  sync::{Arc, AtomicBool},
  web_socket::{
    Frame, FrameMut, OpCode, WebSocketReadMode,
    compression::NegotiatedCompression,
    is_in_continuation_frame::IsInContinuationFrame,
    misc::{write_close_reply, write_control_frame, write_control_frame_cb},
    web_socket_parts::web_socket_part::{WebSocketReaderPart, WebSocketWriterPart},
  },
};
use core::{marker::PhantomData, sync::atomic::Ordering};

/// Owned reader and writer pair
#[derive(Debug)]
pub struct WebSocketPartsOwned<NC, R, SR, SW, const IS_CLIENT: bool> {
  /// Reader
  pub reader: WebSocketReaderPartOwned<NC, R, SR, IS_CLIENT>,
  /// Writer
  pub writer: WebSocketWriterPartOwned<NC, R, SW, IS_CLIENT>,
}

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketReaderPartOwned<NC, R, SR, const IS_CLIENT: bool> {
  pub(crate) connection_state: Arc<AtomicBool>,
  pub(crate) is_in_continuation_frame: Option<IsInContinuationFrame>,
  pub(crate) nc: NC,
  pub(crate) nc_rsv1: u8,
  pub(crate) phantom: PhantomData<SR>,
  pub(crate) rng: R,
  pub(crate) stream_reader: SR,
  pub(crate) wsrp: WebSocketReaderPart<PartitionedFilledBuffer, Vector<u8>, IS_CLIENT>,
}

impl<NC, R, SR, const IS_CLIENT: bool> WebSocketReaderPartOwned<NC, R, SR, IS_CLIENT>
where
  NC: NegotiatedCompression,
  R: Rng,
  SR: StreamReader,
{
  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut Vector<u8>,
    wsrm: WebSocketReadMode,
  ) -> crate::Result<FrameMut<'frame, IS_CLIENT>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    let rslt = self
      .wsrp
      .read_frame_from_parts(
        &mut connection_state,
        &mut self.is_in_continuation_frame,
        &mut self.nc,
        self.nc_rsv1,
        &mut self.rng,
        &mut self.stream_reader,
        buffer,
        wsrm,
      )
      .await?;
    self.connection_state.store(connection_state.into(), Ordering::Relaxed);
    Ok(rslt)
  }
}

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketWriterPartOwned<NC, R, SW, const IS_CLIENT: bool> {
  pub(crate) connection_state: Arc<AtomicBool>,
  pub(crate) nc: NC,
  pub(crate) nc_rsv1: u8,
  pub(crate) rng: R,
  pub(crate) stream_writer: SW,
  pub(crate) wswp: WebSocketWriterPart<Vector<u8>, IS_CLIENT>,
}

impl<NC, R, SW, const IS_CLIENT: bool> WebSocketWriterPartOwned<NC, R, SW, IS_CLIENT>
where
  NC: NegotiatedCompression,
  R: Rng,
  SW: StreamWriter,
{
  /// Should be called when a close frame is received. This method manages state to comply
  /// with the rules stated by the RFC.
  #[inline]
  pub async fn write_close_reply(&mut self, payload: &[u8]) -> crate::Result<()> {
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    let _ = write_close_reply::<_, _, IS_CLIENT>(
      &mut self.stream_writer,
      &mut connection_state,
      self.wswp.no_masking,
      payload,
      &mut self.rng,
      write_control_frame_cb,
    )
    .await?;
    self.connection_state.store(connection_state.into(), Ordering::Relaxed);
    Ok(())
  }

  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, IS_CLIENT>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    self
      .wswp
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

  /// Should be called when a ping frame is received. This method manages state to comply
  /// with the rules stated by the RFC.
  #[inline]
  pub async fn write_ping_reply(&mut self, payload: &[u8]) -> crate::Result<()> {
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    write_control_frame::<_, _, IS_CLIENT>(
      &mut self.stream_writer,
      &mut connection_state,
      self.wswp.no_masking,
      OpCode::Pong,
      payload,
      &mut self.rng,
      |_| {},
      write_control_frame_cb,
    )
    .await?;
    self.connection_state.store(connection_state.into(), Ordering::Relaxed);
    Ok(())
  }
}
