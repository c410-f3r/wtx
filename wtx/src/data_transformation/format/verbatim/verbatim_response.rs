use crate::{
  data_transformation::dnsn::{Deserialize, Serialize},
  misc::Vector,
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("verbatim response")]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
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
  fn seq_from_bytes(_: &mut Vector<Self>, _: &'de [u8], _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl<'de, D, DRSR> Deserialize<'de, &mut DRSR> for VerbatimResponse<D>
where
  VerbatimResponse<D>: Deserialize<'de, DRSR>,
{
  #[inline]
  fn from_bytes(bytes: &'de [u8], drsr: &mut &mut DRSR) -> crate::Result<Self> {
    <VerbatimResponse<D>>::from_bytes(bytes, drsr)
  }

  #[inline]
  fn seq_from_bytes(
    buffer: &mut Vector<Self>,
    bytes: &'de [u8],
    drsr: &mut &mut DRSR,
  ) -> crate::Result<()> {
    <VerbatimResponse<D>>::seq_from_bytes(buffer, bytes, drsr)
  }
}

impl<D> Serialize<()> for VerbatimResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl<D, DRSR> Serialize<&mut DRSR> for VerbatimResponse<D>
where
  VerbatimResponse<D>: Serialize<DRSR>,
{
  #[inline]
  fn to_bytes(&mut self, bytes: &mut Vector<u8>, drsr: &mut &mut DRSR) -> crate::Result<()> {
    self.to_bytes(bytes, drsr)
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::{
    data_transformation::{dnsn::Borsh, format::VerbatimResponse, DataTransformationError},
    misc::Vector,
  };
  use borsh::BorshDeserialize;

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, Borsh> for VerbatimResponse<D>
  where
    D: BorshDeserialize,
  {
    #[inline]
    fn from_bytes(mut bytes: &'de [u8], _: &mut Borsh) -> crate::Result<Self> {
      Ok(Self { data: D::deserialize(&mut bytes)? })
    }

    #[inline]
    fn seq_from_bytes(_: &mut Vector<Self>, _: &'de [u8], _: &mut Borsh) -> crate::Result<()> {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }
}

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::{
    data_transformation::{
      dnsn::{Deserialize, QuickProtobuf, Serialize},
      format::VerbatimResponse,
      DataTransformationError,
    },
    misc::Vector,
  };
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  impl<'de, D> Deserialize<'de, QuickProtobuf> for VerbatimResponse<D>
  where
    D: MessageRead<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &'de [u8], _: &mut QuickProtobuf) -> crate::Result<Self> {
      Ok(Self { data: MessageRead::from_reader(&mut BytesReader::from_bytes(bytes), bytes)? })
    }

    #[inline]
    fn seq_from_bytes(
      _: &mut Vector<Self>,
      _: &'de [u8],
      _: &mut QuickProtobuf,
    ) -> crate::Result<()> {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }

  impl<D> Serialize<QuickProtobuf> for VerbatimResponse<D>
  where
    D: MessageWrite,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut QuickProtobuf) -> crate::Result<()> {
      self.data.write_message(&mut Writer::new(bytes))?;
      Ok(())
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    data_transformation::{
      dnsn::SerdeJson,
      format::{misc::collect_using_serde_json, VerbatimResponse},
    },
    misc::Vector,
  };

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, SerdeJson> for VerbatimResponse<D>
  where
    D: serde::Deserialize<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &'de [u8], _: &mut SerdeJson) -> crate::Result<Self> {
      Ok(Self { data: serde_json::from_slice(bytes)? })
    }

    #[inline]
    fn seq_from_bytes(
      buffer: &mut Vector<Self>,
      bytes: &'de [u8],
      _: &mut SerdeJson,
    ) -> crate::Result<()> {
      collect_using_serde_json(buffer, bytes)
    }
  }

  impl<D> crate::data_transformation::dnsn::Serialize<SerdeJson> for VerbatimResponse<D>
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
