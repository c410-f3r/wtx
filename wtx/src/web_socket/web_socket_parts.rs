use crate::{
  misc::{ConnectionState, FilledBuffer, LeaseMut, PartitionedFilledBuffer, Rng, Stream},
  web_socket::{
    compression::NegotiatedCompression, web_socket_reader, web_socket_writer, Frame, FrameMut,
  },
};
use core::marker::PhantomData;

/// Auxiliary structure used by [`WebSocketReaderStub`] and [`WebSocketWriterStub`]
pub struct WebSocketCommonPart<'instance, NC, RNG, S, const IS_CLIENT: bool> {
  pub(crate) connection_state: &'instance mut ConnectionState,
  pub(crate) nc: &'instance mut NC,
  pub(crate) rng: &'instance mut RNG,
  pub(crate) stream: &'instance mut S,
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
pub struct WebSocketReaderPart<'instance, NC, RNG, S, const IS_CLIENT: bool> {
  pub(crate) max_payload_len: usize,
  pub(crate) network_buffer: &'instance mut PartitionedFilledBuffer,
  pub(crate) phantom: PhantomData<(NC, RNG, S)>,
  pub(crate) reader_buffer_first: &'instance mut FilledBuffer,
  pub(crate) reader_buffer_second: &'instance mut FilledBuffer,
}

impl<'instance, NC, RNG, S, const IS_CLIENT: bool>
  WebSocketReaderPart<'instance, NC, RNG, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
{
  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame(
    &mut self,
    common: &mut WebSocketCommonPart<'instance, NC, RNG, S, IS_CLIENT>,
  ) -> crate::Result<FrameMut<'_, IS_CLIENT>> {
    let WebSocketCommonPart { connection_state, nc, rng, stream } = common;
    let WebSocketReaderPart {
      max_payload_len,
      network_buffer,
      phantom: _,
      reader_buffer_first,
      reader_buffer_second,
    } = self;
    web_socket_reader::read_frame_from_stream(
      connection_state,
      *max_payload_len,
      nc,
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
      rng,
      stream,
    )
    .await
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
pub struct WebSocketWriterPart<'instance, NC, RNG, S, const IS_CLIENT: bool> {
  pub(crate) phantom: PhantomData<(NC, RNG, S)>,
  pub(crate) writer_buffer: &'instance mut FilledBuffer,
}

impl<'instance, NC, RNG, S, const IS_CLIENT: bool>
  WebSocketWriterPart<'instance, NC, RNG, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(
    &mut self,
    common: &mut WebSocketCommonPart<'instance, NC, RNG, S, IS_CLIENT>,
    frame: &mut Frame<P, IS_CLIENT>,
  ) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let WebSocketCommonPart { connection_state, nc, rng, stream } = common;
    web_socket_writer::write_frame(connection_state, frame, nc, rng, stream, self.writer_buffer)
      .await
  }
}
