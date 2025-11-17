use crate::{
  database::client::mysql::{
    misc::packet_header,
    mysql_executor::MAX_PAYLOAD,
    protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  },
  de::Encode,
  misc::Usize,
  stream::Stream,
};
use core::marker::PhantomData;

pub(crate) struct PacketReq<E, T>(pub(crate) T, pub(crate) PhantomData<E>);

impl<E, T> PacketReq<E, T>
where
  E: From<crate::Error>,
  T: Encode<Protocol<(), E>>,
{
  pub(crate) async fn send<S>(
    &self,
    ew: &mut EncodeWrapperProtocol<'_>,
    sequence_id: &mut u8,
    stream: &mut S,
  ) -> Result<(), E>
  where
    S: Stream,
  {
    let mut chunks = ew.encode_buffer.chunks_exact(*Usize::from(MAX_PAYLOAD));
    for chunk in chunks.by_ref() {
      let header = packet_header(chunk.len(), sequence_id);
      stream.write_all_vectored(&[header.as_slice(), chunk]).await?;
    }
    let chunk = chunks.remainder();
    let header = packet_header(chunk.len(), sequence_id);
    stream.write_all_vectored(&[header.as_slice(), chunk]).await?;
    Ok(())
  }
}
