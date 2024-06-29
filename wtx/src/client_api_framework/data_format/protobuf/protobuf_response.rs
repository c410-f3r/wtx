use crate::client_api_framework::dnsn::{Deserialize, Serialize};
use alloc::vec::Vec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Protobuf response")]
pub struct ProtobufResponse<D> {
  /// Actual data
  pub data: D,
}

impl<D> Deserialize<()> for ProtobufResponse<D>
where
  D: Default,
{
  #[inline]
  fn from_bytes(_: &[u8], _: &mut ()) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }

  #[inline]
  fn seq_from_bytes<E>(_: &[u8], _: &mut (), _: impl FnMut(Self) -> Result<(), E>) -> Result<(), E>
  where
    E: From<crate::Error>,
  {
    Ok(())
  }
}

impl<D> Serialize<()> for ProtobufResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "protobuf")]
mod protobuf {
  use crate::client_api_framework::{
    data_format::ProtobufResponse, dnsn::Protobuf, ClientApiFrameworkError,
  };
  use core::fmt::Display;
  use protobuf::Message;

  impl<D> crate::client_api_framework::dnsn::Deserialize<Protobuf> for ProtobufResponse<D>
  where
    D: Message,
  {
    fn from_bytes(bytes: &[u8], _: &mut Protobuf) -> crate::Result<Self> {
      Ok(Self { data: Message::parse_from_bytes(bytes)? })
    }

    fn seq_from_bytes<E>(
      _: &[u8],
      _: &mut Protobuf,
      _: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: Display + From<crate::Error>,
    {
      Err(E::from(ClientApiFrameworkError::UnsupportedOperation.into()))
    }
  }
}
