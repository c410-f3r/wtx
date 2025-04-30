use crate::{
  collection::Vector,
  misc::{ConnectionState, LeaseMut, net::PartitionedFilledBuffer},
  rng::Rng,
  stream::Stream,
  web_socket::{
    Frame, FrameMut,
    compression::NegotiatedCompression,
    web_socket_parts::web_socket_part::{
      WebSocketCommonPart, WebSocketReaderPart, WebSocketWriterPart,
    },
  },
};
use core::marker::PhantomData;

/// Auxiliary structure used by [`WebSocketReaderPartMut`] and [`WebSocketWriterPartMut`]
#[derive(Debug)]
pub struct WebSocketCommonPartMut<'instance, NC, R, S, const IS_CLIENT: bool> {
  pub(crate) wsc: WebSocketCommonPart<
    &'instance mut ConnectionState,
    &'instance mut NC,
    &'instance mut R,
    &'instance mut S,
    IS_CLIENT,
  >,
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketReaderPartMut<'instance, NC, R, S, const IS_CLIENT: bool> {
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
  pub async fn read_frame(
    &mut self,
    common: &mut WebSocketCommonPartMut<'instance, NC, R, S, IS_CLIENT>,
  ) -> crate::Result<FrameMut<'_, IS_CLIENT>> {
    self.wsrp.read_frame_from_stream(&mut common.wsc).await
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
    self.wswp.write_frame(&mut common.wsc, frame).await
  }
}
