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
  fn seq_from_bytes<E>(
    _: &'de [u8],
    _: &mut (),
    _: impl FnMut(Self) -> Result<(), E>,
  ) -> Result<(), E>
  where
    E: From<crate::Error>,
  {
    Ok(())
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
    data_transformation::{dnsn::SerdeJson, format::JsonResponse, seq_visitor::_SeqVisitor},
    misc::Vector,
  };
  use core::fmt::Display;
  use serde::de::Deserializer;

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, SerdeJson> for JsonResponse<D>
  where
    D: serde::Deserialize<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &'de [u8], _: &mut SerdeJson) -> crate::Result<Self> {
      Ok(JsonResponse { data: serde_json::from_slice(bytes)? })
    }

    #[inline]
    fn seq_from_bytes<E>(
      bytes: &'de [u8],
      _: &mut SerdeJson,
      mut cb: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: Display + From<crate::Error>,
    {
      let mut de = serde_json::Deserializer::from_slice(bytes);
      de.deserialize_seq(_SeqVisitor::_new(|data| cb(Self { data }))).map_err(Into::into)?;
      Ok(())
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
