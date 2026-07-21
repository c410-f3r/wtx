use crate::{
  database::{
    DatabaseError,
    client::postgres::{
      PostgresClient,
      message::{Message, MessageTy},
    },
  },
  misc::Usize,
  net::{BufStreamReader, ConnectionState, Stream},
  tls::{TlsMode, TlsStream},
};

impl<E, S, TM> PostgresClient<E, S, TM>
where
  S: Stream,
  TM: TlsMode,
{
  pub(crate) async fn fetch_msg<'nb>(
    cs: &mut ConnectionState,
    read_buffer: &'nb mut BufStreamReader,
    stream: &mut TlsStream<S, TM, true>,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg(read_buffer, stream).await?;
    Ok(Message { tag, ty: MessageTy::try_from((cs, tag, read_buffer.current()))? })
  }

  async fn fetch_representative_msg(
    read_buffer: &mut BufStreamReader,
    stream: &mut TlsStream<S, TM, true>,
  ) -> crate::Result<u8> {
    let mut tag = Self::fetch_single_msg(&mut *read_buffer, stream).await?;
    while tag == b'N' {
      tag = Self::fetch_single_msg(read_buffer, stream).await?;
    }
    Ok(tag)
  }

  // | Ty | Len | Payload |
  // | 1  |  4  |    x    |
  //
  // The value of `Len` is payload length plus 4, therefore, the frame length is `Len` plus 1.
  async fn fetch_single_msg(
    read_buffer: &mut BufStreamReader,
    stream: &mut TlsStream<S, TM, true>,
  ) -> crate::Result<u8> {
    let [b0, b1, b2, b3, b4] =
      read_buffer.read_header::<_, 5>(stream).await?.ok_or(DatabaseError::AbruptDisconnect)?;
    let len = Usize::from(u32::from_be_bytes([b1, b2, b3, b4])).into_usize().wrapping_sub(4);
    read_buffer.read_payload(len, stream).await?;
    Ok(b0)
  }
}
