use crate::{
  database::{
    DatabaseError,
    client::postgres::{
      PostgresClient,
      message::{Message, MessageTy},
    },
  },
  misc::Usize,
  net::{BufStreamReader, Stream},
  tls::{TlsCtx, TlsStream},
};
use core::hint::cold_path;

const MAX_IGNORABLE_MESSAGES_AFTER_ERR: u8 = 5;

impl<E, S, TCX> PostgresClient<E, S, TCX>
where
  S: Stream,
  TCX: TlsCtx,
{
  pub(crate) async fn fetch_msg<'nb>(
    read_buffer: &'nb mut BufStreamReader,
    stream: &mut TlsStream<S, TCX, true>,
  ) -> crate::Result<Message<'nb>> {
    let tag = Self::fetch_representative_msg(read_buffer, stream).await?;
    if tag == b'E' {
      cold_path();
      'err: {
        for _ in 0..MAX_IGNORABLE_MESSAGES_AFTER_ERR {
          let local_tag = Self::fetch_single_msg(read_buffer, stream).await?;
          if local_tag == b'Z' {
            break 'err;
          }
        }
        return Err(DatabaseError::MaxIgnorableMessages.into());
      }
    }
    Ok(Message { tag, ty: MessageTy::try_from((tag, read_buffer.current()))? })
  }

  async fn fetch_representative_msg(
    read_buffer: &mut BufStreamReader,
    stream: &mut TlsStream<S, TCX, true>,
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
    stream: &mut TlsStream<S, TCX, true>,
  ) -> crate::Result<u8> {
    let [b0, b1, b2, b3, b4] =
      read_buffer.read_header::<_, 5>(stream).await?.ok_or(DatabaseError::AbruptDisconnect)?;
    let len = Usize::from(u32::from_be_bytes([b1, b2, b3, b4])).into_usize().wrapping_sub(4);
    read_buffer.read_payload(len, stream).await?;
    Ok(b0)
  }
}
