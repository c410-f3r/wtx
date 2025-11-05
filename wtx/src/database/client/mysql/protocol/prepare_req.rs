use crate::{
  database::client::mysql::protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  de::Encode,
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
    let _ = ew.encode_buffer.extend_from_copyable_slices([&[22], self.query])?;
    Ok(())
  }
}
