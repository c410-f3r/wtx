// Common functions that used be used by pure WebSocket structures or tunneling protocols.

use crate::{
  collection::{ExpansionTy, Vector},
  misc::{ConnectionState, Lease, LeaseMut},
  rng::Rng,
  stream::StreamWriter,
  web_socket::{
    Frame, FrameMut, OpCode, compression::NegotiatedCompression, misc::has_masked_frame,
    unmask::unmask,
  },
};

pub(crate) fn manage_compression<NC, P, const IS_CLIENT: bool>(
  frame: &mut Frame<P, IS_CLIENT>,
  nc: &NC,
) -> bool
where
  NC: NegotiatedCompression,
  P: Lease<[u8]>,
{
  if NC::IS_NOOP {
    return false;
  }
  let mut should_compress = false;
  if !frame.op_code().is_control() {
    let [first, _] = frame.header_first_two_mut();
    should_compress = nc.rsv1() != 0;
    *first |= nc.rsv1();
  }
  should_compress
}

pub(crate) fn manage_frame_compression<'cb, NC, P, R, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  nc: &mut NC,
  frame: &mut Frame<P, IS_CLIENT>,
  no_masking: bool,
  rng: &mut R,
  writer_buffer: &'cb mut Vector<u8>,
) -> crate::Result<FrameMut<'cb, IS_CLIENT>>
where
  NC: NegotiatedCompression,
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if frame.op_code() == OpCode::Close {
    *connection_state = ConnectionState::Closed;
  }
  let mut compressed_frame = compress_frame(frame, nc, writer_buffer)?;
  mask_frame(&mut compressed_frame, no_masking, rng);
  Ok(compressed_frame)
}

pub(crate) fn manage_normal_frame<P, R, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P, IS_CLIENT>,
  no_masking: bool,
  rng: &mut R,
) where
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if frame.op_code() == OpCode::Close {
    *connection_state = ConnectionState::Closed;
  }
  mask_frame(frame, no_masking, rng);
}

pub(crate) async fn write_frame<NC, P, R, SW, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P, IS_CLIENT>,
  no_masking: bool,
  nc: &mut NC,
  rng: &mut R,
  stream: &mut SW,
  writer_buffer: &mut Vector<u8>,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
  P: LeaseMut<[u8]>,
  R: Rng,
  SW: StreamWriter,
{
  if manage_compression(frame, nc) {
    let fr = manage_frame_compression(connection_state, nc, frame, no_masking, rng, writer_buffer)?;
    stream.write_all_vectored(&[fr.header(), fr.payload()]).await?;
  } else {
    manage_normal_frame::<_, _, IS_CLIENT>(connection_state, frame, no_masking, rng);
    let (header, payload) = frame.header_and_payload_mut();
    stream.write_all_vectored(&[header, payload.lease()]).await?;
  }
  Ok(())
}

fn compress_frame<'cb, P, NC, const IS_CLIENT: bool>(
  frame: &mut Frame<P, IS_CLIENT>,
  nc: &mut NC,
  writer_buffer: &'cb mut Vector<u8>,
) -> crate::Result<FrameMut<'cb, IS_CLIENT>>
where
  P: LeaseMut<[u8]>,
  NC: NegotiatedCompression,
{
  let additional = frame.payload().lease().len().wrapping_add(128);
  writer_buffer.clear();
  let mut payload_len = nc.compress(
    frame.payload().lease(),
    writer_buffer,
    |local_writer_buffer| {
      local_writer_buffer.expand(ExpansionTy::Additional(additional), 0)?;
      Ok(local_writer_buffer)
    },
    |local_writer_buffer, written| {
      local_writer_buffer.expand(ExpansionTy::Additional(additional), 0)?;
      Ok(local_writer_buffer.get_mut(written..).unwrap_or_default())
    },
  )?;
  if frame.fin() {
    payload_len = payload_len.saturating_sub(4);
  }
  Ok(FrameMut::new(
    frame.fin(),
    frame.op_code(),
    writer_buffer.get_mut(..payload_len).unwrap_or_default(),
    nc.rsv1(),
  ))
}

fn mask_frame<P, R, const IS_CLIENT: bool>(
  frame: &mut Frame<P, IS_CLIENT>,
  no_masking: bool,
  rng: &mut R,
) where
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if IS_CLIENT && !no_masking && !has_masked_frame(*frame.header_first_two_mut()[1]) {
    let mask: [u8; 4] = rng.u8_4();
    frame.set_mask(mask);
    unmask(frame.payload_mut().lease_mut(), mask);
  }
}
