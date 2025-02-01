use crate::{data_transformation::dnsn::Serialize, misc::Vector};

/// A wrapper for data types that don't require a special pre-fixed structure.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VerbatimRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for VerbatimRequest<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl<D, DRSR> Serialize<&mut DRSR> for VerbatimRequest<D>
where
  VerbatimRequest<D>: Serialize<DRSR>,
{
  #[inline]
  fn to_bytes(&mut self, bytes: &mut Vector<u8>, drsr: &mut &mut DRSR) -> crate::Result<()> {
    self.to_bytes(bytes, drsr)
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::{
    data_transformation::{dnsn::Borsh, format::VerbatimRequest},
    misc::Vector,
  };
  use borsh::{BorshDeserialize, BorshSerialize};

  impl<'de, D> crate::data_transformation::dnsn::Deserialize<'de, Borsh> for VerbatimRequest<D>
  where
    D: BorshDeserialize,
  {
    #[inline]
    fn from_bytes(mut bytes: &'de [u8], _: &mut Borsh) -> crate::Result<Self> {
      Ok(Self { data: D::deserialize(&mut bytes)? })
    }

    #[inline]
    fn seq_from_bytes(_: &mut Vector<Self>, _: &'de [u8], _: &mut Borsh) -> crate::Result<()> {
      Ok(())
    }
  }

  impl<D> crate::data_transformation::dnsn::Serialize<Borsh> for VerbatimRequest<D>
  where
    D: BorshSerialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut Borsh) -> crate::Result<()> {
      self.data.serialize(bytes)?;
      Ok(())
    }
  }
}

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::{
    data_transformation::{
      dnsn::{Deserialize, QuickProtobuf, Serialize},
      format::VerbatimRequest,
      DataTransformationError,
    },
    misc::Vector,
  };
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  impl<'de, D> Deserialize<'de, QuickProtobuf> for VerbatimRequest<D>
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

  impl<D> Serialize<QuickProtobuf> for VerbatimRequest<D>
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
    data_transformation::{dnsn::SerdeJson, format::VerbatimRequest},
    misc::Vector,
  };

  impl<D> crate::data_transformation::dnsn::Serialize<SerdeJson> for VerbatimRequest<D>
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
