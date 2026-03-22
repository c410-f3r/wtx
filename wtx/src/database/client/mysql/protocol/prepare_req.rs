use crate::{
  codec::Encode,
  database::client::mysql::{
    command::Command,
    protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  },
};

pub(crate) struct PrepareReq<'any> {
  pub(crate) query: &'any [u8],
}

impl<E> Encode<Protocol<(), E>> for PrepareReq<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let _ = ew
      .encode_buffer
      .extend_from_copyable_slices([&[Command::ComStmtPrepare.into()], self.query])?;
    Ok(())
  }
}
