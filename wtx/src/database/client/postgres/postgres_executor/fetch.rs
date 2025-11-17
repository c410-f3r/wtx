use crate::{
  database::client::postgres::{
    ExecutorBuffer, PostgresExecutor,
    message::{Message, MessageTy},
  },
  misc::{
    ConnectionState, LeaseMut, Usize,
    net::{PartitionedFilledBuffer, read_header, read_payload},
  },
  stream::Stream,
};

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn fetch_msg_from_stream<'nb>(
    cs: &mut ConnectionState,
    net_buffer: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg_from_stream(net_buffer, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from((cs, net_buffer.current()))? })
  }

  // | Ty | Len | Payload |
  // | 1  |  4  |    x    |
  //
  // The value of `Len` is payload length plus 4, therefore, the frame length is `Len` plus 1.
  async fn fetch_one_msg_from_stream(
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    net_buffer.reserve(5)?;
    let mut read = net_buffer.following_len();
    let buffer = net_buffer.following_rest_mut();
    let [a, b, c, d, e] = read_header::<0, 5, S>(buffer, &mut read, stream).await?;
    let len = Usize::from(u32::from_be_bytes([b, c, d, e])).into_usize().wrapping_add(1);
    read_payload((0, len), net_buffer, &mut read, stream).await?;
    Ok(a)
  }

  async fn fetch_representative_msg_from_stream(
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut tag = Self::fetch_one_msg_from_stream(&mut *net_buffer, stream).await?;
    while tag == b'N' {
      tag = Self::fetch_one_msg_from_stream(net_buffer, stream).await?;
    }
    Ok(tag)
  }
}
