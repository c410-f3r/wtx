use crate::client_api_framework::dnsn::Serialize;
use alloc::vec::Vec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("JSON request")]
pub struct JsonRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for JsonRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "miniserde")]
mod miniserde {
  use crate::client_api_framework::{
    data_format::JsonRequest,
    dnsn::{miniserde_serialize, Miniserde},
  };
  use alloc::vec::Vec;

  impl<D> crate::client_api_framework::dnsn::Serialize<Miniserde> for JsonRequest<D>
  where
    D: miniserde::Serialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut Miniserde) -> crate::Result<()> {
      if core::mem::size_of::<D>() == 0 {
        return Ok(());
      }
      miniserde_serialize(bytes, &self.data)
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::client_api_framework::{data_format::JsonRequest, dnsn::SerdeJson};
  use alloc::vec::Vec;

  impl<D> crate::client_api_framework::dnsn::Serialize<SerdeJson> for JsonRequest<D>
  where
    D: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      if core::mem::size_of::<D>() == 0 {
        return Ok(());
      }
      serde_json::to_writer(bytes, &self.data)?;
      Ok(())
    }
  }
}

#[cfg(feature = "simd-json")]
mod simd_json {
  use crate::client_api_framework::{data_format::JsonRequest, dnsn::SimdJson};
  use alloc::vec::Vec;

  impl<D> crate::client_api_framework::dnsn::Serialize<SimdJson> for JsonRequest<D>
  where
    D: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SimdJson) -> crate::Result<()> {
      if core::mem::size_of::<D>() == 0 {
        return Ok(());
      }
      simd_json::to_writer(bytes, &self.data)?;
      Ok(())
    }
  }
}
