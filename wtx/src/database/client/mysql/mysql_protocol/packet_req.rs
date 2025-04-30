use core::marker::PhantomData;

use crate::{
  database::client::mysql::{
    mysql_executor::MAX_PAYLOAD,
    mysql_protocol::{MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  },
  misc::{Encode, Usize},
  stream::Stream,
};

pub(crate) struct PacketReq<E, T>(pub(crate) T, pub(crate) PhantomData<E>);

impl<E, T> PacketReq<E, T>
where
  E: From<crate::Error>,
  T: Encode<MysqlProtocol<(), E>>,
{
  #[inline]
  pub(crate) async fn encode_and_write<S>(
    &self,
    ew: &mut EncodeWrapperProtocol<'_>,
    sequence_id: &mut u8,
    stream: &mut S,
  ) -> Result<(), E>
  where
    S: Stream,
  {
    let copy_into_header = |len: usize, local_sequence_id: &mut u8| {
      let mut len_u32 = u32::try_from(len).unwrap_or_default().to_le_bytes();
      len_u32[3] = *local_sequence_id;
      *local_sequence_id = local_sequence_id.wrapping_add(1);
      len_u32
    };
    self.0.encode(&mut (), ew)?;
    let mut chunks = ew.encode_buffer.chunks_exact(*Usize::from(MAX_PAYLOAD));
    for chunk in chunks.by_ref() {
      let len = copy_into_header(chunk.len(), sequence_id);
      stream.write_all_vectored(&[len.as_slice(), chunk]).await?;
    }
    let remainder = chunks.remainder();
    let len = copy_into_header(remainder.len(), sequence_id);
    stream.write_all_vectored(&[len.as_slice(), remainder]).await?;
    Ok(())
  }
}
