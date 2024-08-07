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

#[cfg(feature = "serde_yaml")]
mod serde_yaml {
  use crate::{
    client_api_framework::{data_format::YamlRequest, dnsn::SerdeYaml},
    misc::Vector,
  };

  impl<D> crate::client_api_framework::dnsn::Serialize<SerdeYaml> for YamlRequest<D>
  where
    D: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SerdeYaml) -> crate::Result<()> {
      if size_of::<D>() == 0 {
        return Ok(());
      }
      serde_yaml::to_writer(bytes, &self.data)?;
      Ok(())
    }
  }
}
