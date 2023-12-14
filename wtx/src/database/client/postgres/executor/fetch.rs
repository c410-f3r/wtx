use crate::{
  database::client::postgres::{message::Message, Executor, ExecutorBuffer, MessageTy},
  misc::{PartitionedFilledBuffer, Stream, _read_until},
};
use core::borrow::BorrowMut;

impl<EB, S> Executor<EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn fetch_msg_from_stream<'nb>(
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg_from_stream(nb, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from(nb._current())? })
  }

  async fn fetch_one_header_from_stream(
    nb: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<(u8, usize)> {
    let buffer = nb._following_trail_mut();
    let [mt_n, b, c, d, e] = _read_until::<5, S>(buffer, read, 0, stream).await?;
    let len: usize = u32::from_be_bytes([b, c, d, e]).try_into()?;
    Ok((mt_n, len.wrapping_add(1)))
  }

  async fn fetch_one_msg_from_stream<'nb>(
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut read = nb._following_len();
    let (ty, len) = Self::fetch_one_header_from_stream(nb, &mut read, stream).await?;
    Self::fetch_one_payload_from_stream(len, nb, &mut read, stream).await?;
    let current_end_idx = nb._current_end_idx();
    nb._set_indices(current_end_idx, len, read.wrapping_sub(len))?;
    Ok(ty)
  }

  async fn fetch_one_payload_from_stream(
    len: usize,
    nb: &mut PartitionedFilledBuffer,
    read: &mut usize,
    stream: &mut S,
  ) -> crate::Result<()> {
    let mut is_payload_filled = false;
    nb._expand_following(len);
    for _ in 0..len {
      if *read >= len {
        is_payload_filled = true;
        break;
      }
      *read = read.wrapping_add(
        stream.read(nb._following_trail_mut().get_mut(*read..).unwrap_or_default()).await?,
      );
    }
    if !is_payload_filled {
      return Err(crate::Error::UnexpectedBufferState);
    }
    Ok(())
  }

  pub(crate) async fn fetch_representative_msg_from_stream<'nb>(
    nb: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut tag = Self::fetch_one_msg_from_stream(&mut *nb, stream).await?;
    if tag == b'N' {
      tag = Self::fetch_one_msg_from_stream(nb, stream).await?;
    }
    Ok(tag)
  }
}
