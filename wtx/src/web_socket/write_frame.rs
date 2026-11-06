// Common functions that used be used by pure WebSocket structures or tunneling protocols.

use crate::{
  codec::CompressionFlush,
  collections::{ArrayVectorCopy, ArrayVectorU8, VectorUsize},
  misc::{Lease, LeaseMut},
  net::{MAX_VECTORED_WRITES, StreamWriter},
  rng::Rng,
  web_socket::{
    Frame, MASK_MASK, MAX_HEADER_LEN,
    mask_op::mask_op,
    misc::{has_masked_frame, header_from_params},
    web_socket_compression::WebSocketCompression,
  },
};
use core::hint::cold_path;

pub(crate) fn mask_frame<P, R, const IS_CLIENT: bool>(
  frame: &mut Frame<P>,
  no_masking: bool,
  rng: &mut R,
) where
  P: LeaseMut<[u8]>,
  R: Rng,
{
  if IS_CLIENT && !no_masking && !has_masked_frame(*frame.header_first_two_mut()[1]) {
    let mask: [u8; 4] = rng.u8_4();
    frame.set_mask(mask);
    mask_op(frame.payload_mut().lease_mut(), mask);
  }
}

pub(crate) async fn write_frame<C, P, R, SW, const IS_CLIENT: bool>(
  frame: &mut Frame<P>,
  no_masking: bool,
  nc: &mut C,
  nc_rsv1: u8,
  rng: &mut R,
  stream_writer: &mut SW,
  writer_buffer: &mut VectorUsize<u8>,
  closed_conn_cb: impl FnOnce(&mut SW),
) -> crate::Result<()>
where
  C: WebSocketCompression,
  P: LeaseMut<[u8]>,
  R: Rng,
  SW: StreamWriter,
{
  if manage_compression::<C, _>(frame, nc_rsv1) {
    writer_buffer.clear();
    let mut cf = compress_frame(frame, nc, nc_rsv1, writer_buffer)?;
    mask_compressed_frame::<_, IS_CLIENT>(&mut cf, no_masking, rng, writer_buffer);
    let header = cf.header.as_slice();
    let payload = writer_buffer.get(cf.begin..cf.end).unwrap_or_default();
    stream_writer.write_all_vectored(&[header, payload]).await?;
  } else {
    mask_frame::<_, _, IS_CLIENT>(frame, no_masking, rng);
    let (header, payload) = frame.header_and_payload_mut();
    stream_writer.write_all_vectored(&[header, payload.lease()]).await?;
  }
  if frame.op_code().is_close() {
    cold_path();
    closed_conn_cb(stream_writer);
  }
  Ok(())
}

pub(crate) async fn write_frames<'frame, C, P, R, SW, const IS_CLIENT: bool>(
  frames: impl IntoIterator<Item = &'frame mut Frame<P>>,
  nc_rsv1: u8,
  nc: &mut C,
  no_masking: bool,
  rng: &mut R,
  stream_writer: &mut SW,
  writer_buffer: &mut VectorUsize<u8>,
  mut closed_conn_cb: impl FnMut(&mut SW),
) -> crate::Result<()>
where
  C: WebSocketCompression,
  P: LeaseMut<[u8]> + 'frame,
  R: Rng,
  SW: StreamWriter,
{
  const FRAMES_PER_BATCH: usize = MAX_VECTORED_WRITES / 2;

  let mut iter = frames.into_iter();
  loop {
    let mut batch = ArrayVectorU8::<_, FRAMES_PER_BATCH>::new();
    let mut has_close = false;

    writer_buffer.clear();
    // `writer_buffer` has *****mutable***** operations
    while usize::from(batch.len()) < FRAMES_PER_BATCH {
      let Some(frame) = iter.next() else { break };
      let cf = if manage_compression::<C, _>(frame, nc_rsv1) {
        let mut cf = compress_frame(frame, nc, nc_rsv1, writer_buffer)?;
        mask_compressed_frame::<_, IS_CLIENT>(&mut cf, no_masking, rng, writer_buffer);
        Some(cf)
      } else {
        mask_frame::<_, _, IS_CLIENT>(frame, no_masking, rng);
        None
      };
      has_close |= frame.op_code().is_close();
      let _rslt = batch.push(PreparedFrame { cf, frame });
    }
    if batch.is_empty() {
      break;
    }
    let mut slices = ArrayVectorCopy::<_, MAX_VECTORED_WRITES>::new();
    // `writer_buffer` has *****immutable***** operations
    for prepared in &batch {
      let (header, payload) = if let Some(cf) = &prepared.cf {
        (cf.header.as_slice(), writer_buffer.get(cf.begin..cf.end).unwrap_or_default())
      } else {
        let (header, payload) = prepared.frame.header_and_payload();
        (header, payload.lease())
      };
      let _rslt = slices.extend_from_copyable_slice(&[header, payload.lease()]);
    }
    stream_writer.write_all_vectored(slices.as_slice()).await?;
    if has_close {
      cold_path();
      closed_conn_cb(stream_writer);
    }
  }
  Ok(())
}

struct CompressedFrame {
  begin: usize,
  end: usize,
  header: ArrayVectorCopy<u8, MAX_HEADER_LEN>,
}

struct PreparedFrame<'frame, P> {
  cf: Option<CompressedFrame>,
  frame: &'frame mut Frame<P>,
}

#[inline]
fn compress_frame<C, P>(
  frame: &Frame<P>,
  nc: &mut C,
  nc_rsv1: u8,
  writer_buffer: &mut VectorUsize<u8>,
) -> crate::Result<CompressedFrame>
where
  C: WebSocketCompression,
  P: Lease<[u8]>,
{
  let begin = writer_buffer.len();
  let input = frame.payload().lease();
  let mut payload_len = nc.compress(CompressionFlush::SyncFlush, input, writer_buffer)?;
  if frame.fin() {
    if !C::IS_NOOP && nc.no_context_takeover() {
      nc.reset();
    }
    payload_len = payload_len.saturating_sub(4);
  }
  let end = begin.wrapping_add(payload_len);
  writer_buffer.truncate(end);
  Ok(CompressedFrame {
    begin,
    end,
    header: header_from_params(frame.fin(), frame.op_code(), payload_len, nc_rsv1),
  })
}

#[inline]
fn mask_compressed_frame<R, const IS_CLIENT: bool>(
  compressed: &mut CompressedFrame,
  no_masking: bool,
  rng: &mut R,
  writer_buffer: &mut VectorUsize<u8>,
) where
  R: Rng,
{
  if IS_CLIENT
    && !no_masking
    && !has_masked_frame(compressed.header.get(1).copied().unwrap_or_default())
  {
    let mask: [u8; 4] = rng.u8_4();
    let [_, second, ..] = compressed.header.as_slice_mut() else {
      return;
    };
    *second |= MASK_MASK;
    let _rslt = compressed.header.extend_from_copyable_slice(&mask);
    let range = compressed.begin..compressed.end;
    mask_op(writer_buffer.get_mut(range).unwrap_or_default(), mask);
  }
}

#[inline]
fn manage_compression<C, P>(frame: &mut Frame<P>, nc_rsv1: u8) -> bool
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
