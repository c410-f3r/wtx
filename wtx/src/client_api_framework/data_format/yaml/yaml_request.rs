use crate::{client_api_framework::dnsn::Serialize, misc::Vector};

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("YAML request")]
pub struct YamlRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for YamlRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}
