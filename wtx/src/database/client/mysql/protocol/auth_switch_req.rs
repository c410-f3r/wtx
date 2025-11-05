use crate::{
  database::client::mysql::protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  de::Encode,
};

pub(crate) struct AuthSwitchReq<'bytes>(pub(crate) &'bytes [u8]);

impl<E> Encode<Protocol<(), E>> for AuthSwitchReq<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    ew.encode_buffer.extend_from_copyable_slice(self.0)?;
    Ok(())
  }
}
