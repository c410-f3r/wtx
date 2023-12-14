use crate::client_api_framework::dnsn::Serialize;
use alloc::vec::Vec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Protobuf request")]
pub struct ProtobufRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for ProtobufRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "protobuf")]
mod protobuf {
  use crate::client_api_framework::{data_format::ProtobufRequest, dnsn::Protobuf};
  use alloc::vec::Vec;
  use protobuf::Message;

  impl<D> crate::client_api_framework::dnsn::Serialize<Protobuf> for ProtobufRequest<D>
  where
    D: Message,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut Protobuf) -> crate::Result<()> {
      self.data.write_to_writer(bytes)?;
      Ok(())
    }
  }
}
