// Common functions that used be used by pure WebSocket structures or tunneling protocols.

use crate::{
  codec::CompressionFlush,
  collections::Vector,
  misc::{ConnectionState, Lease, LeaseMut},
  rng::Rng,
  stream::StreamWriter,
  web_socket::{
    Frame, FrameMut, OpCode, misc::has_masked_frame, unmask::unmask,
    web_socket_compression::WebSocketCompression,
  },
};

pub(crate) fn manage_compression<C, P>(frame: &mut Frame<P>, nc_rsv1: u8) -> bool
where
  C: WebSocketCompression,
  P: Lease<[u8]>,
{
  if C::IS_NOOP {
    return false;
  }
  let mut should_compress = false;
  if !frame.op_code().is_control() {
    let [first, _] = frame.header_first_two_mut();
    should_compress = nc_rsv1 != 0;
    *first |= nc_rsv1;
  }
  should_compress
}

pub(crate) fn manage_frame_compression<'cb, C, P, R, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  nc: &mut C,
  nc_rsv1: u8,
  frame: &mut Frame<P>,
  no_masking: bool,
  rng: &mut R,
  writer_buffer: &'cb mut Vector<u8>,
) -> crate::Result<FrameMut<'cb>>
where
  C: WebSocketCompression,
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if frame.op_code() == OpCode::Close {
    *connection_state = ConnectionState::Closed;
  }
  let mut compressed_frame = compress_frame(frame, nc, nc_rsv1, writer_buffer)?;
  mask_frame::<_, _, IS_CLIENT>(&mut compressed_frame, no_masking, rng);
  Ok(compressed_frame)
}

pub(crate) fn manage_normal_frame<P, R, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P>,
  no_masking: bool,
  rng: &mut R,
) where
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if frame.op_code() == OpCode::Close {
    *connection_state = ConnectionState::Closed;
  }
  mask_frame::<_, _, IS_CLIENT>(frame, no_masking, rng);
}

pub(crate) async fn write_frame<C, P, R, SW, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P>,
  no_masking: bool,
  nc: &mut C,
  nc_rsv1: u8,
  rng: &mut R,
  stream_writer: &mut SW,
  writer_buffer: &mut Vector<u8>,
) -> crate::Result<()>
where
  C: WebSocketCompression,
  P: LeaseMut<[u8]>,
  R: Rng,
  SW: StreamWriter,
{
  if manage_compression::<C, _>(frame, nc_rsv1) {
    let fr = manage_frame_compression::<_, _, _, IS_CLIENT>(
      connection_state,
      nc,
      nc_rsv1,
      frame,
      no_masking,
      rng,
      writer_buffer,
    )?;
    stream_writer.write_all_vectored(&[fr.header(), fr.payload()]).await?;
  } else {
    manage_normal_frame::<_, _, IS_CLIENT>(connection_state, frame, no_masking, rng);
    let (header, payload) = frame.header_and_payload_mut();
    stream_writer.write_all_vectored(&[header, payload.lease()]).await?;
  }
  Ok(())
}

fn compress_frame<'cb, C, P>(
  frame: &mut Frame<P>,
  nc: &mut C,
  nc_rsv1: u8,
  writer_buffer: &'cb mut Vector<u8>,
) -> crate::Result<FrameMut<'cb>>
where
  C: WebSocketCompression,
  P: LeaseMut<[u8]>,
{
  writer_buffer.clear();
  let bytes = frame.payload().lease();
  let mut payload_len = nc.compress(CompressionFlush::SyncFlush, bytes, writer_buffer)?;
  if frame.fin() {
    if !C::IS_NOOP && nc.no_context_takeover() {
      nc.reset();
    }
    payload_len = payload_len.saturating_sub(4);
  }
  Ok(FrameMut::new(
    frame.fin(),
    frame.op_code(),
    writer_buffer.get_mut(..payload_len).unwrap_or_default(),
    nc_rsv1,
  ))
}

fn mask_frame<P, R, const IS_CLIENT: bool>(frame: &mut Frame<P>, no_masking: bool, rng: &mut R)
where
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if IS_CLIENT && !no_masking && !has_masked_frame(*frame.header_first_two_mut()[1]) {
    let mask: [u8; 4] = rng.u8_4();
    frame.set_mask(mask);
    unmask(frame.payload_mut().lease_mut(), mask);
  }
}
