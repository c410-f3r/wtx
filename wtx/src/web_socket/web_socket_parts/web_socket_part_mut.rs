use crate::{
  collection::Vector,
  misc::{ConnectionState, LeaseMut, net::PartitionedFilledBuffer},
  rng::Rng,
  stream::Stream,
  web_socket::{
    Frame, FrameMut, WebSocketReadFrameTy,
    compression::NegotiatedCompression,
    is_in_continuation_frame::IsInContinuationFrame,
    web_socket_parts::web_socket_part::{WebSocketReaderPart, WebSocketWriterPart},
  },
};
use core::marker::PhantomData;

/// Auxiliary structure used by [`WebSocketReaderPartMut`] and [`WebSocketWriterPartMut`]
#[derive(Debug)]
pub struct WebSocketCommonPartMut<'instance, NC, R, S, const IS_CLIENT: bool> {
  pub(crate) connection_state: &'instance mut ConnectionState,
  pub(crate) nc: &'instance mut NC,
  pub(crate) rng: &'instance mut R,
  pub(crate) stream: &'instance mut S,
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketReaderPartMut<'instance, NC, R, S, const IS_CLIENT: bool> {
  pub(crate) is_in_continuation_frame: &'instance mut Option<IsInContinuationFrame>,
  pub(crate) phantom: PhantomData<(NC, R, S)>,
  pub(crate) wsrp: WebSocketReaderPart<
    &'instance mut PartitionedFilledBuffer,
    &'instance mut Vector<u8>,
    IS_CLIENT,
  >,
}

impl<'instance, NC, R, S, const IS_CLIENT: bool>
  WebSocketReaderPartMut<'instance, NC, R, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  R: Rng,
  S: Stream,
{
  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame<'buffer, 'frame, 'this>(
    &'this mut self,
    buffer: &'buffer mut Vector<u8>,
    common: &mut WebSocketCommonPartMut<'instance, NC, R, S, IS_CLIENT>,
  ) -> crate::Result<(FrameMut<'frame, IS_CLIENT>, WebSocketReadFrameTy)>
  where
    'buffer: 'frame,
    'this: 'frame,
  {
    self
      .wsrp
      .read_frame_from_stream(
        common.connection_state,
        self.is_in_continuation_frame,
        common.nc,
        common.rng,
        common.stream,
        buffer,
      )
      .await
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketWriterPartMut<'instance, NC, R, S, const IS_CLIENT: bool> {
  pub(crate) phantom: PhantomData<(NC, R, S)>,
  pub(crate) wswp: WebSocketWriterPart<&'instance mut Vector<u8>, IS_CLIENT>,
}

impl<'instance, NC, R, S, const IS_CLIENT: bool>
  WebSocketWriterPartMut<'instance, NC, R, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  R: Rng,
  S: Stream,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(
    &mut self,
    common: &mut WebSocketCommonPartMut<'instance, NC, R, S, IS_CLIENT>,
    frame: &mut Frame<P, IS_CLIENT>,
  ) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    self
      .wswp
      .write_frame(common.connection_state, frame, common.nc, common.rng, common.stream)
      .await
  }
}
