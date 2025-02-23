use crate::{
  database::client::mysql::mysql_protocol::{
    MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol,
  },
  misc::Encode,
};

pub(crate) struct PrepareReq<'any> {
  pub(crate) query: &'any [u8],
}

impl<E> Encode<MysqlProtocol<(), E>> for PrepareReq<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let _ = ew.enc_buffer.extend_from_copyable_slices([&[22], self.query])?;
    Ok(())
  }
}
