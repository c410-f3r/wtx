use crate::{
  misc::{
    partitioned_filled_buffer::PartitionedFilledBuffer, ConnectionState, LeaseMut, Lock,
    StreamReader, StreamWriter, Vector, Xorshift64,
  },
  web_socket::{
    compression::NegotiatedCompression,
    payload_ty::PayloadTy,
    web_socket_parts::web_socket_part::{
      WebSocketCommonPart, WebSocketReaderPart, WebSocketWriterPart,
    },
    Frame, FrameMut,
  },
};
use core::marker::PhantomData;

/// Owned reader and writer pair
#[derive(Debug)]
pub struct WebSocketPartsOwned<C, NC, SR, SW, const IS_CLIENT: bool> {
  /// Reader
  pub reader: WebSocketReaderPartOwned<C, NC, SR, IS_CLIENT>,
  /// Writer
  pub writer: WebSocketWriterPartOwned<C, NC, SW, IS_CLIENT>,
}

/// Auxiliary structure used by [`WebSocketReaderPartOwned`] and [`WebSocketWriterPartOwned`]
#[derive(Debug)]
pub struct WebSocketCommonPartOwned<NC, SW, const IS_CLIENT: bool> {
  pub(crate) wsc: WebSocketCommonPart<ConnectionState, NC, Xorshift64, SW, IS_CLIENT>,
}

/// Reader that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketReaderPartOwned<C, NC, SR, const IS_CLIENT: bool> {
  pub(crate) common: C,
  pub(crate) phantom: PhantomData<(NC, SR)>,
  pub(crate) stream_reader: SR,
  pub(crate) wsrp: WebSocketReaderPart<PartitionedFilledBuffer, PayloadTy, Vector<u8>, IS_CLIENT>,
}

impl<C, NC, SR, SW, const IS_CLIENT: bool> WebSocketReaderPartOwned<C, NC, SR, IS_CLIENT>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, IS_CLIENT>>,
  NC: NegotiatedCompression,
  SR: StreamReader,
  SW: StreamWriter,
{
  /// The current frame payload that is set when [`Self::read_frame`] is called, otherwise,
  /// returns an empty slice.
  #[inline]
  pub fn curr_payload(&mut self) -> &mut [u8] {
    match self.wsrp.curr_payload {
      PayloadTy::Network => self.wsrp.network_buffer._current_mut(),
      PayloadTy::None => &mut [],
      PayloadTy::FirstReader => self.wsrp.reader_buffer_first.as_slice_mut(),
      PayloadTy::SecondReader => self.wsrp.reader_buffer_second.as_slice_mut(),
    }
  }

  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame(&mut self) -> crate::Result<FrameMut<'_, IS_CLIENT>> {
    self.wsrp.read_frame_from_parts(&mut self.common, &mut self.stream_reader).await
  }
}

/// Writer that can be used in concurrent scenarios.
#[derive(Debug)]
pub struct WebSocketWriterPartOwned<C, NC, SW, const IS_CLIENT: bool> {
  pub(crate) common: C,
  pub(crate) phantom: PhantomData<(NC, SW)>,
  pub(crate) wswp: WebSocketWriterPart<Vector<u8>, IS_CLIENT>,
}

impl<C, NC, SW, const IS_CLIENT: bool> WebSocketWriterPartOwned<C, NC, SW, IS_CLIENT>
where
  C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, IS_CLIENT>>,
  NC: NegotiatedCompression,
  SW: StreamWriter,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(&mut self, frame: &mut Frame<P, IS_CLIENT>) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    self.wswp.write_frame(&mut self.common.lock().await.wsc, frame).await
  }
}
