use crate::client_api_framework::dnsn::Serialize;
use alloc::vec::Vec;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("verbatim request")]
pub struct VerbatimRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for VerbatimRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "rkyv")]
mod rkyv {
  use crate::client_api_framework::{
    data_format::VerbatimRequest,
    dnsn::{Rkyv, _InnerSerializer},
  };

  impl<D> crate::client_api_framework::dnsn::Serialize<Rkyv> for VerbatimRequest<D>
  where
    for<'any> D: rkyv::Serialize<_InnerSerializer<'any>>,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, drsr: &mut Rkyv) -> crate::Result<()> {
      drsr._serialize(bytes, &self.data)?;
      Ok(())
    }
  }
}
