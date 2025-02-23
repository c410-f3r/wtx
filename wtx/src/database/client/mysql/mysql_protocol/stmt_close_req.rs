use crate::{
  database::client::mysql::mysql_protocol::{
    MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol,
  },
  misc::Encode,
};

#[derive(Debug)]
pub(crate) struct StmtCloseReq {
  pub(crate) statement: u32,
}

impl<E> Encode<MysqlProtocol<(), E>> for StmtCloseReq
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let array = [&[25][..], &self.statement.to_le_bytes()];
    let _ = ew.enc_buffer.extend_from_copyable_slices(array)?;
    Ok(())
  }
}
