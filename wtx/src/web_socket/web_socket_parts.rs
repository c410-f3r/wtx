use crate::{
  misc::{ConnectionState, LeaseMut, PartitionedFilledBuffer, Stream, Vector, Xorshift64},
  web_socket::{
    compression::NegotiatedCompression, payload_ty::PayloadTy, web_socket_reader,
    web_socket_writer, Frame, FrameMut,
  },
};
use core::marker::PhantomData;

/// Auxiliary structure used by [`WebSocketReaderStub`] and [`WebSocketWriterStub`]
#[derive(Debug)]
pub struct WebSocketCommonPart<'instance, NC, S, const IS_CLIENT: bool> {
  pub(crate) connection_state: &'instance mut ConnectionState,
  pub(crate) curr_payload: &'instance mut PayloadTy,
  pub(crate) nc: &'instance mut NC,
  pub(crate) rng: &'instance mut Xorshift64,
  pub(crate) stream: &'instance mut S,
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketReaderPart<'instance, NC, S, const IS_CLIENT: bool> {
  pub(crate) max_payload_len: usize,
  pub(crate) network_buffer: &'instance mut PartitionedFilledBuffer,
  pub(crate) phantom: PhantomData<(NC, S)>,
  pub(crate) reader_buffer_first: &'instance mut Vector<u8>,
  pub(crate) reader_buffer_second: &'instance mut Vector<u8>,
}

impl<'instance, NC, S, const IS_CLIENT: bool> WebSocketReaderPart<'instance, NC, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  S: Stream,
{
  /// Reads a frame from the stream.
  ///
  /// If a frame is made up of other sub-frames or continuations, then everything is collected
  /// until all fragments are received.
  #[inline]
  pub async fn read_frame(
    &mut self,
    common: &mut WebSocketCommonPart<'instance, NC, S, IS_CLIENT>,
  ) -> crate::Result<FrameMut<'_, IS_CLIENT>> {
    let WebSocketCommonPart { connection_state, curr_payload, nc, rng, stream } = common;
    let Self {
      max_payload_len,
      network_buffer,
      phantom: _,
      reader_buffer_first,
      reader_buffer_second,
    } = self;
    let (frame, payload_ty) = web_socket_reader::read_frame_from_stream(
      connection_state,
      *max_payload_len,
      nc,
      network_buffer,
      reader_buffer_first,
      reader_buffer_second,
      rng,
      stream,
    )
    .await?;
    **curr_payload = payload_ty;
    Ok(frame)
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub struct WebSocketWriterPart<'instance, NC, S, const IS_CLIENT: bool> {
  pub(crate) phantom: PhantomData<(NC, S)>,
  pub(crate) writer_buffer: &'instance mut Vector<u8>,
}

impl<'instance, NC, S, const IS_CLIENT: bool> WebSocketWriterPart<'instance, NC, S, IS_CLIENT>
where
  NC: NegotiatedCompression,
  S: Stream,
{
  /// Writes a frame to the stream.
  #[inline]
  pub async fn write_frame<P>(
    &mut self,
    common: &mut WebSocketCommonPart<'instance, NC, S, IS_CLIENT>,
    frame: &mut Frame<P, IS_CLIENT>,
  ) -> crate::Result<()>
  where
    P: LeaseMut<[u8]>,
  {
    let WebSocketCommonPart { connection_state, curr_payload: _, nc, rng, stream } = common;
    let Self { phantom: _, writer_buffer } = self;
    web_socket_writer::write_frame(connection_state, frame, nc, rng, stream, writer_buffer).await?;
    Ok(())
  }
}
