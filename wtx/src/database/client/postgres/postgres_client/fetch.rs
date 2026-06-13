use crate::{
  database::client::postgres::{
    ClientBuffer, PostgresClient,
    message::{Message, MessageTy},
  },
  misc::{
    ConnectionState, LeaseMut, PartitionedFilledBuffer, Usize,
    net::{read_header, read_payload},
  },
  stream::Stream,
};

impl<CB, E, S> PostgresClient<CB, E, S>
where
  CB: LeaseMut<ClientBuffer>,
  S: Stream,
{
  pub(crate) async fn fetch_msg_from_stream<'nb>(
    cs: &mut ConnectionState,
    read_buffer: &'nb mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg_from_stream(read_buffer, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from((cs, read_buffer.current()))? })
  }

  // | Ty | Len | Payload |
  // | 1  |  4  |    x    |
  //
  // The value of `Len` is payload length plus 4, therefore, the frame length is `Len` plus 1.
  async fn fetch_one_msg_from_stream(
    read_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    read_buffer.reserve(5)?;
    let mut read = read_buffer.following_len();
    let buffer = read_buffer.following_rest_mut();
    let [a, b, c, d, e] = read_header::<0, 5, S>(buffer, &mut read, stream).await?;
    let len = Usize::from(u32::from_be_bytes([b, c, d, e])).into_usize().wrapping_add(1);
    read_payload((0, len), read_buffer, &mut read, stream).await?;
    Ok(a)
  }

  async fn fetch_representative_msg_from_stream(
    read_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<u8> {
    let mut tag = Self::fetch_one_msg_from_stream(&mut *read_buffer, stream).await?;
    while tag == b'N' {
      tag = Self::fetch_one_msg_from_stream(read_buffer, stream).await?;
    }
    Ok(tag)
  }
}
