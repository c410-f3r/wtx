use crate::{
  collection::Vector,
  misc::{ConnectionState, LeaseMut, net::PartitionedFilledBuffer},
  rng::Rng,
  stream::{Stream, StreamReader, StreamWriter},
  web_socket::{
    Frame, FrameMut, WebSocketPayloadOrigin, compression::NegotiatedCompression,
    is_in_continuation_frame::IsInContinuationFrame, web_socket_reader::read_frame,
    web_socket_replier::WebSocketReplier, web_socket_writer,
  },
};

#[derive(Debug)]
pub(crate) struct WebSocketReaderGeneric<PFB, V, const IS_CLIENT: bool> {
  pub(crate) max_payload_len: usize,
  pub(crate) network_buffer: PFB,
  pub(crate) no_masking: bool,
  pub(crate) reader_buffer: V,
}

impl<PFB, V, const IS_CLIENT: bool> WebSocketReaderGeneric<PFB, V, IS_CLIENT>
where
  PFB: LeaseMut<PartitionedFilledBuffer>,
  V: LeaseMut<Vector<u8>>,
{
  pub(crate) async fn read_frame_mut<'frame, 'this, 'ub, NC, R, S>(
    &'this mut self,
    connection_state: &mut ConnectionState,
    is_in_continuation_frame: &mut Option<IsInContinuationFrame>,
    nc: &mut NC,
    nc_rsv1: u8,
    payload_origin: WebSocketPayloadOrigin,
    rng: &mut R,
    stream: &mut S,
    user_buffer: &'ub mut Vector<u8>,
  ) -> crate::Result<FrameMut<'frame, IS_CLIENT>>
  where
    'this: 'frame,
    'ub: 'frame,
    NC: NegotiatedCompression,
    R: Rng,
    S: Stream,
  {
    let Self { max_payload_len, network_buffer, no_masking, reader_buffer } = self;
    read_frame::<_, _, _, _, _, true, IS_CLIENT>(
      connection_state,
      is_in_continuation_frame,
      *max_payload_len,
      nc,
      nc_rsv1,
      network_buffer.lease_mut(),
      *no_masking,
      payload_origin,
      reader_buffer.lease_mut(),
      &WebSocketReplier::new(),
      rng,
      stream,
      user_buffer,
      |local_stream| local_stream,
      |local_stream| local_stream,
    )
    .await
  }

  pub(crate) async fn read_frame_owned<'frame, 'this, 'ub, NC, R, SR>(
    &'this mut self,
    connection_state: &mut ConnectionState,
    is_in_continuation_frame: &mut Option<IsInContinuationFrame>,
    nc: &mut NC,
    nc_rsv1: u8,
    payload_origin: WebSocketPayloadOrigin,
    replier: &WebSocketReplier<IS_CLIENT>,
    rng: &mut R,
    stream_reader: &mut SR,
    user_buffer: &'ub mut Vector<u8>,
  ) -> crate::Result<FrameMut<'frame, IS_CLIENT>>
  where
    'this: 'frame,
    'ub: 'frame,
    NC: NegotiatedCompression,
    R: Rng,
    SR: StreamReader,
  {
    let Self { max_payload_len, network_buffer, no_masking, reader_buffer } = self;
    read_frame::<_, _, _, _, _, false, IS_CLIENT>(
      connection_state,
      is_in_continuation_frame,
      *max_payload_len,
      nc,
      nc_rsv1,
      network_buffer.lease_mut(),
      *no_masking,
      payload_origin,
      reader_buffer.lease_mut(),
      replier,
      rng,
      &mut (stream_reader, &mut ()),
      user_buffer,
      |local_stream| local_stream.0,
      |local_stream| local_stream.1,
    )
    .await
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub(crate) struct WebSocketWriterGeneric<V, const IS_CLIENT: bool> {
  pub(crate) no_masking: bool,
  pub(crate) writer_buffer: V,
}

impl<V, const IS_CLIENT: bool> WebSocketWriterGeneric<V, IS_CLIENT>
where
  V: LeaseMut<Vector<u8>>,
{
  pub(crate) async fn write_frame<NC, P, R, SW>(
    &mut self,
    connection_state: &mut ConnectionState,
    frame: &mut Frame<P, IS_CLIENT>,
    nc: &mut NC,
    nc_rsv1: u8,
    rng: &mut R,
    stream_writer: &mut SW,
  ) -> crate::Result<()>
  where
    NC: NegotiatedCompression,
    P: LeaseMut<[u8]>,
    R: Rng,
    SW: StreamWriter,
  {
    let Self { no_masking, writer_buffer } = self;
    web_socket_writer::write_frame(
      connection_state,
      frame,
      *no_masking,
      nc,
      nc_rsv1,
      rng,
      stream_writer,
      writer_buffer.lease_mut(),
    )
    .await?;
    Ok(())
  }
}
