use crate::{
  database::client::mysql::protocol::{Protocol, encode_wrapper_protocol::EncodeWrapperProtocol},
  de::Encode,
};

pub(crate) struct QueryReq<'any> {
  pub(crate) query: &'any [u8],
}

impl<E> Encode<Protocol<(), E>> for QueryReq<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    let _ = ew.encode_buffer.extend_from_copyable_slices([&[3], self.query])?;
    Ok(())
  }
}
