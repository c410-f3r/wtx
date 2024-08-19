use crate::{
  data_transformation::dnsn::{Deserialize, Serialize},
  misc::Vector,
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("verbatim response")]
pub struct VerbatimResponse<D> {
  /// Actual data
  pub data: D,
}

impl<'de, D> Deserialize<'de, ()> for VerbatimResponse<D>
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

impl<D> Serialize<()> for VerbatimResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "rkyv")]
mod rkyv {
  use crate::data_transformation::{dnsn::Rkyv, format::VerbatimResponse, DataTransformationError};
  use core::fmt::Display;
  use rkyv::{
    bytecheck::CheckBytes, de::deserializers::SharedDeserializeMap,
    validation::validators::DefaultValidator, Archive,
  };

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, Rkyv> for VerbatimResponse<D>
  where
    D: Archive,
    for<'any> D::Archived:
      CheckBytes<DefaultValidator<'any>> + rkyv::Deserialize<D, SharedDeserializeMap>,
  {
    fn from_bytes(bytes: &[u8], _: &mut Rkyv) -> crate::Result<Self> {
      Ok(Self {
        data: rkyv::from_bytes(bytes)
          .map_err(|_err| crate::Error::RkyvDer(core::any::type_name::<D>()))?,
      })
    }

    fn seq_from_bytes<E>(
      _: &[u8],
      _: &mut Rkyv,
      _: impl FnMut(Self) -> Result<(), E>,
    ) -> Result<(), E>
    where
      E: Display + From<crate::Error>,
    {
      Err(E::from(DataTransformationError::UnsupportedOperation.into()))
    }
  }
}
