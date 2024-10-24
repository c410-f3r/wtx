use crate::{
  misc::{BufferMode, ConnectionState, Lease, LeaseMut, Rng, Stream, Vector, Xorshift64},
  web_socket::{compression::NegotiatedCompression, unmask::unmask, Frame, FrameMut, OpCode},
};

#[inline]
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

#[inline]
pub(crate) fn manage_frame_compression<'cb, P, NC, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  nc: &mut NC,
  frame: &mut Frame<P, IS_CLIENT>,
  rng: &mut Xorshift64,
  writer_buffer: &'cb mut Vector<u8>,
) -> crate::Result<FrameMut<'cb, IS_CLIENT>>
where
  P: LeaseMut<[u8]>,
  NC: NegotiatedCompression,
{
  if frame.op_code() == OpCode::Close {
    *connection_state = ConnectionState::Closed;
  }
  let mut compressed_frame = compress_frame(frame, nc, writer_buffer)?;
  mask_frame(&mut compressed_frame, rng);
  Ok(compressed_frame)
}

#[inline]
pub(crate) fn manage_normal_frame<P, RNG, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P, IS_CLIENT>,
  rng: &mut RNG,
) where
  P: LeaseMut<[u8]>,
  RNG: Rng,
{
  if frame.op_code() == OpCode::Close {
    *connection_state = ConnectionState::Closed;
  }
  mask_frame(frame, rng);
}

#[inline]
pub(crate) async fn write_frame<NC, P, S, const IS_CLIENT: bool>(
  connection_state: &mut ConnectionState,
  frame: &mut Frame<P, IS_CLIENT>,
  nc: &mut NC,
  rng: &mut Xorshift64,
  stream: &mut S,
  writer_buffer: &mut Vector<u8>,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
  P: LeaseMut<[u8]>,
  S: Stream,
{
  if manage_compression(frame, nc) {
    let nframe = manage_frame_compression(connection_state, nc, frame, rng, writer_buffer)?;
    stream.write_all_vectored(&[nframe.header(), nframe.payload()]).await?;
  } else {
    manage_normal_frame::<_, _, IS_CLIENT>(connection_state, frame, rng);
    let (header, payload) = frame.header_and_payload_mut();
    stream.write_all_vectored(&[header, payload.lease()]).await?;
  }
  Ok(())
}

#[inline]
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
      local_writer_buffer.expand(BufferMode::Additional(additional), 0)?;
      Ok(local_writer_buffer)
    },
    |local_writer_buffer, written| {
      local_writer_buffer.expand(BufferMode::Additional(additional), 0)?;
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

#[inline]
const fn has_masked_frame(second_header_byte: u8) -> bool {
  second_header_byte & 0b1000_0000 != 0
}

#[inline]
fn mask_frame<P, RNG, const IS_CLIENT: bool>(frame: &mut Frame<P, IS_CLIENT>, rng: &mut RNG)
where
  P: LeaseMut<[u8]>,
  RNG: Rng,
{
  if IS_CLIENT {
    if let [_, second_byte, .., a, b, c, d] = frame.header_mut() {
      if !has_masked_frame(*second_byte) {
        *second_byte |= 0b1000_0000;
        let mask = rng.u8_4();
        *a = mask[0];
        *b = mask[1];
        *c = mask[2];
        *d = mask[3];
        unmask(frame.payload_mut().lease_mut(), mask);
      }
    }
  }
}
