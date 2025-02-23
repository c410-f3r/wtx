use crate::{
  database::client::mysql::mysql_protocol::{
    MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol,
  },
  misc::Encode,
};

pub(crate) struct AuthSwitchReq<'bytes>(pub(crate) &'bytes [u8]);

impl<E> Encode<MysqlProtocol<(), E>> for AuthSwitchReq<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    ew.enc_buffer.extend_from_copyable_slice(self.0)?;
    Ok(())
  }
}
