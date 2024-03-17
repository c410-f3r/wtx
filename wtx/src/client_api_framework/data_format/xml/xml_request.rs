use crate::client_api_framework::dnsn::Serialize;
use alloc::vec::Vec;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("XML request")]
pub struct XmlRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for XmlRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "serde-xml-rs")]
mod serde_xml_rs {
  use crate::client_api_framework::{data_format::XmlRequest, dnsn::SerdeXmlRs};
  use alloc::vec::Vec;

  impl<D> crate::client_api_framework::dnsn::Serialize<SerdeXmlRs> for XmlRequest<D>
  where
    D: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SerdeXmlRs) -> crate::Result<()> {
      if core::mem::size_of::<D>() == 0 {
        return Ok(());
      }
      serde_xml_rs::to_writer(bytes, &self.data)?;
      Ok(())
    }
  }
}
