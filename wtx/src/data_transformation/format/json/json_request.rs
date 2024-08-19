use crate::{data_transformation::dnsn::Serialize, misc::Vector};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("JSON request")]
pub struct JsonRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for JsonRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    data_transformation::{dnsn::SerdeJson, format::JsonRequest},
    misc::Vector,
  };

  impl<D> crate::data_transformation::dnsn::Serialize<SerdeJson> for JsonRequest<D>
  where
    D: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      if size_of::<D>() == 0 {
        return Ok(());
      }
      serde_json::to_writer(bytes, &self.data)?;
      Ok(())
    }
  }
}
