use crate::{
  data_transformation::dnsn::{Deserialize, Serialize},
  misc::Vector,
};

#[doc = generic_data_format_doc!("JSON response")]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct JsonResponse<D> {
  /// Actual data
  pub data: D,
}

impl<'de, D> Deserialize<'de, ()> for JsonResponse<D>
where
  D: Default,
{
  #[inline]
  fn from_bytes(_: &'de [u8], _: &mut ()) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }

  #[inline]
  fn seq_from_bytes(_: &'de [u8], _: &mut ()) -> impl Iterator<Item = crate::Result<Self>> {
    [].into_iter()
  }
}

impl<D> Serialize<()> for JsonResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    data_transformation::{dnsn::SerdeJson, format::JsonResponse},
    misc::Vector,
  };
  use serde_json::{de::SliceRead, StreamDeserializer};

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, SerdeJson> for JsonResponse<D>
  where
    D: serde::Deserialize<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &'de [u8], _: &mut SerdeJson) -> crate::Result<Self> {
      Ok(JsonResponse { data: serde_json::from_slice(bytes)? })
    }

    #[inline]
    fn seq_from_bytes(
      bytes: &'de [u8],
      _: &mut SerdeJson,
    ) -> impl Iterator<Item = crate::Result<Self>> {
      StreamDeserializer::new(SliceRead::new(bytes))
        .map(|el| el.map(|data| JsonResponse { data }).map_err(From::from))
    }
  }

  impl<D> crate::data_transformation::dnsn::Serialize<SerdeJson> for JsonResponse<D>
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
