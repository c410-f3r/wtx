use crate::{client_api_framework::dnsn::Serialize, misc::Vector};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Protobuf request")]
pub struct ProtobufRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for ProtobufRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "protobuf")]
mod protobuf {
  use crate::{
    client_api_framework::{data_format::ProtobufRequest, dnsn::Protobuf},
    misc::Vector,
  };
  use protobuf::Message;

  impl<D> crate::client_api_framework::dnsn::Serialize<Protobuf> for ProtobufRequest<D>
  where
    D: Message,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut Protobuf) -> crate::Result<()> {
      self.data.write_to_writer(bytes)?;
      Ok(())
    }
  }
}
