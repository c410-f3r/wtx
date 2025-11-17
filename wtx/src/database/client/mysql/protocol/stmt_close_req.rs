use crate::{
  database::client::mysql::{
    command::Command,
    protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  },
  de::Encode,
};

#[derive(Debug)]
pub(crate) struct StmtCloseReq {
  pub(crate) statement: u32,
}

impl<E> Encode<Protocol<(), E>> for StmtCloseReq
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let array = [&[Command::ComStmtClose.into()][..], &self.statement.to_le_bytes()];
    let _ = ew.encode_buffer.extend_from_copyable_slices(array)?;
    Ok(())
  }
}
