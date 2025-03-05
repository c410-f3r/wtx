use crate::{
  misc::{
    ConnectionState, LeaseMut, Lock, Stream, StreamReader, StreamWriter, Vector, Xorshift64,
    partitioned_filled_buffer::PartitionedFilledBuffer,
  },
  web_socket::{
    Frame, FrameMut, compression::NegotiatedCompression,
    web_socket_parts::web_socket_part_owned::WebSocketCommonPartOwned, web_socket_writer,
  },
};

#[derive(Debug)]
pub(crate) struct WebSocketCommonPart<CS, NC, RNG, S, const IS_CLIENT: bool> {
  pub(crate) connection_state: CS,
  pub(crate) nc: NC,
  pub(crate) rng: RNG,
  pub(crate) stream: S,
}

#[derive(Debug)]
pub(crate) struct WebSocketReaderPart<PFB, V, const IS_CLIENT: bool> {
  pub(crate) max_payload_len: usize,
  pub(crate) nc_rsv1: u8,
  pub(crate) network_buffer: PFB,
  pub(crate) no_masking: bool,
  pub(crate) reader_buffer_first: V,
  pub(crate) reader_buffer_second: V,
}

impl<PFB, V, const IS_CLIENT: bool> WebSocketReaderPart<PFB, V, IS_CLIENT>
where
  PFB: LeaseMut<PartitionedFilledBuffer>,
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  pub(crate) async fn read_frame_from_stream<CS, NC, RNG, S>(
    &mut self,
    common: &mut WebSocketCommonPart<CS, NC, RNG, S, IS_CLIENT>,
  ) -> crate::Result<FrameMut<'_, IS_CLIENT>>
  where
    CS: LeaseMut<ConnectionState>,
    NC: NegotiatedCompression,
    RNG: LeaseMut<Xorshift64>,
    S: Stream,
  {
    let WebSocketCommonPart { connection_state, nc, rng, stream } = common;
    let Self {
      max_payload_len,
      nc_rsv1,
      network_buffer,
      no_masking,
      reader_buffer_first,
      reader_buffer_second,
    } = self;
    let frame = read_frame!(
      *max_payload_len,
      (NC::IS_NOOP, *nc_rsv1),
      network_buffer.lease_mut(),
      *no_masking,
      reader_buffer_first.lease_mut(),
      reader_buffer_second.lease_mut(),
      stream,
      (
        stream,
        WebSocketCommonPart::<_, _, _, _, IS_CLIENT> {
          connection_state: &mut *connection_state,
          nc: &mut *nc,
          rng: &mut *rng,
          stream: &mut *stream
        }
      )
    );
    Ok(frame)
  }

  #[inline]
  pub(crate) async fn read_frame_from_parts<C, NC, SR, SW>(
    &mut self,
    common: &mut C,
    stream_reader: &mut SR,
  ) -> crate::Result<FrameMut<'_, IS_CLIENT>>
  where
    C: Lock<Resource = WebSocketCommonPartOwned<NC, SW, IS_CLIENT>>,
    NC: NegotiatedCompression,
    SR: StreamReader,
    SW: StreamWriter,
  {
    let Self {
      max_payload_len,
      network_buffer,
      nc_rsv1,
      no_masking,
      reader_buffer_first,
      reader_buffer_second,
    } = self;
    let parts = &mut (stream_reader, common);
    let frame = read_frame!(
      *max_payload_len,
      (NC::IS_NOOP, *nc_rsv1),
      network_buffer.lease_mut(),
      *no_masking,
      reader_buffer_first.lease_mut(),
      reader_buffer_second.lease_mut(),
      parts,
      (&mut parts.0, &mut parts.1.lock().await.wsc)
    );
    Ok(frame)
  }
}

/// Auxiliary structure that can be used when it is necessary to write a received frame that belongs
/// to the same instance.
#[derive(Debug)]
pub(crate) struct WebSocketWriterPart<V, const IS_CLIENT: bool> {
  pub(crate) no_masking: bool,
  pub(crate) writer_buffer: V,
}

impl<V, const IS_CLIENT: bool> WebSocketWriterPart<V, IS_CLIENT>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  pub(crate) async fn write_frame<CS, NC, P, RNG, SW>(
    &mut self,
    common: &mut WebSocketCommonPart<CS, NC, RNG, SW, IS_CLIENT>,
    frame: &mut Frame<P, IS_CLIENT>,
  ) -> crate::Result<()>
  where
    CS: LeaseMut<ConnectionState>,
    NC: NegotiatedCompression,
    P: LeaseMut<[u8]>,
    RNG: LeaseMut<Xorshift64>,
    SW: StreamWriter,
  {
    let WebSocketCommonPart { connection_state, nc, rng, stream } = common;
    let Self { no_masking, writer_buffer } = self;
    web_socket_writer::write_frame(
      connection_state.lease_mut(),
      frame,
      *no_masking,
      nc,
      rng.lease_mut(),
      stream,
      writer_buffer.lease_mut(),
    )
    .await?;
    Ok(())
  }
}
