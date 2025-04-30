mod partitioned_filled_buffer;

use crate::stream::StreamReader;
pub(crate) use partitioned_filled_buffer::PartitionedFilledBuffer;

#[inline]
pub(crate) async fn read_header<const BEGIN: usize, const LEN: usize, SR>(
  buffer: &mut [u8],
  read: &mut usize,
  stream_reader: &mut SR,
) -> crate::Result<[u8; LEN]>
where
  [u8; LEN]: Default,
  SR: StreamReader,
{
  loop {
    let (lhs, rhs) = buffer.split_at_mut_checked(*read).unwrap_or_default();
    if let Some(slice) = lhs.get(BEGIN..BEGIN.wrapping_add(LEN)) {
      return Ok(slice.try_into().unwrap_or_default());
    }
    let local_read = stream_reader.read(rhs).await?;
    if local_read == 0 {
      return Err(crate::Error::ClosedConnection);
    }
    *read = read.wrapping_add(local_read);
  }
}

#[inline]
pub(crate) async fn read_payload<SR>(
  (header_len, payload_len): (usize, usize),
  pfb: &mut PartitionedFilledBuffer,
  read: &mut usize,
  stream: &mut SR,
) -> crate::Result<()>
where
  SR: StreamReader,
{
  let frame_len = header_len.wrapping_add(payload_len);
  pfb._reserve(frame_len)?;
  loop {
    if *read >= frame_len {
      break;
    }
    let local_buffer = pfb._following_rest_mut().get_mut(*read..).unwrap_or_default();
    let local_read = stream.read(local_buffer).await?;
    if local_read == 0 {
      return Err(crate::Error::ClosedConnection);
    }
    *read = read.wrapping_add(local_read);
  }
  pfb._set_indices(
    pfb._current_end_idx().wrapping_add(header_len),
    payload_len,
    read.wrapping_sub(frame_len),
  )?;
  Ok(())
}
