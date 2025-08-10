use crate::{
  collection::Vector,
  misc::{LeaseMut, net::PartitionedFilledBuffer},
  rng::Rng,
  stream::{StreamReader, StreamWriter},
  sync::{Arc, AtomicBool},
  web_socket::{
    Frame, FrameControlArray, FrameMut, WebSocketReadMode,
    compression::NegotiatedCompression,
    is_in_continuation_frame::IsInContinuationFrame,
    web_socket_parts::web_socket_part::{WebSocketReaderPart, WebSocketWriterPart},
    web_socket_reply_manager::WebSocketReplyManager,
  },
};
use core::{marker::PhantomData, sync::atomic::Ordering};

/// Owned reader and writer pair
#[derive(Debug)]
pub struct WebSocketPartsOwned<NC, R, SR, SW, const IS_CLIENT: bool> {
  /// See [`WebSocketReaderPartOwned`];
  pub reader: WebSocketReaderPartOwned<NC, R, SR, IS_CLIENT>,
  /// See [`WebSocketReplyManager`];
  pub reply_manager: Arc<WebSocketReplyManager<IS_CLIENT>>,
  /// See [`WebSocketWriterPartOwned`];
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
  pub(crate) reader_part: WebSocketReaderPart<PartitionedFilledBuffer, Vector<u8>, IS_CLIENT>,
  pub(crate) reply_manager: Arc<WebSocketReplyManager<IS_CLIENT>>,
  pub(crate) rng: R,
  pub(crate) stream_reader: SR,
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
    read_mode: WebSocketReadMode,
  ) -> crate::Result<FrameMut<'frame, IS_CLIENT>>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    let mut connection_state = self.connection_state.load(Ordering::Relaxed).into();
    let rslt = self
      .reader_part
      .read_frame_from_owned_parts(
        &mut connection_state,
        &mut self.is_in_continuation_frame,
        &mut self.nc,
        self.nc_rsv1,
        read_mode,
        &self.reply_manager,
        &mut self.rng,
        &mut self.stream_reader,
        buffer,
      )
      .await?;
    self.connection_state.store(connection_state.into(), Ordering::Relaxed);
    Ok(rslt)
  }
}

impl<NC, R, SR, const IS_CLIENT: bool> Drop for WebSocketReaderPartOwned<NC, R, SR, IS_CLIENT> {
  #[inline]
  fn drop(&mut self) {
    let _rslt = self.reply_manager.data().fetch_update(|elem| Some((true, elem.1)));
    self.reply_manager.waker().wake();
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
  pub(crate) writer_part: WebSocketWriterPart<Vector<u8>, IS_CLIENT>,
}

impl<NC, R, SW, const IS_CLIENT: bool> WebSocketWriterPartOwned<NC, R, SW, IS_CLIENT>
where
  NC: NegotiatedCompression,
  R: Rng,
  SW: StreamWriter,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, IS_CLIENT>) -> crate::Result<()>
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

  /// Awaits until a control frame is returned by [`WebSocketReplyManager::reply_frame`] and then
  /// writes it back to the stream. Returns `true` if the connection has been closed.
  #[inline]
  pub async fn write_reply_frame(
    &mut self,
    control_frame: &mut Option<FrameControlArray<IS_CLIENT>>,
  ) -> crate::Result<bool> {
    match control_frame {
      Some(frame) => {
        self.write_frame(frame).await?;
        if frame.op_code().is_close() {
          return Ok(true);
        }
      }
      None => return Ok(true),
    }
    Ok(false)
  }
}
