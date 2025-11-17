use crate::{
  database::client::mysql::{
    command::Command,
    protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  },
  de::Encode,
};

pub(crate) struct PingReq;

impl<E> Encode<Protocol<(), E>> for PingReq
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    ew.encode_buffer.push(Command::ComPing.into())?;
    Ok(())
  }
}
