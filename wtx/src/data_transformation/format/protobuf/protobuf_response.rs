use crate::{
  data_transformation::dnsn::{Deserialize, Serialize},
  misc::Vector,
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Protobuf response")]
pub struct ProtobufResponse<D> {
  /// Actual data
  pub data: D,
}

impl<'de, D> Deserialize<'de, ()> for ProtobufResponse<D>
where
  D: Default,
{
  #[inline]
  fn from_bytes(_: &[u8], _: &mut ()) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }

  #[inline]
  fn seq_from_bytes(_: &'de [u8], _: &mut ()) -> impl Iterator<Item = crate::Result<Self>> {
    [].into_iter()
  }
}

impl<D> Serialize<()> for ProtobufResponse<D> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::{
    data_transformation::{
      dnsn::{Deserialize, QuickProtobuf, Serialize},
      format::ProtobufResponse,
    },
    misc::Vector,
  };
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  impl<'de, D> Deserialize<'de, QuickProtobuf> for ProtobufResponse<D>
  where
    D: MessageRead<'de>,
  {
    #[inline]
    fn from_bytes(bytes: &'de [u8], _: &mut QuickProtobuf) -> crate::Result<Self> {
      Ok(Self { data: MessageRead::from_reader(&mut BytesReader::from_bytes(bytes), bytes)? })
    }

    #[inline]
    fn seq_from_bytes(
      _: &'de [u8],
      _: &mut QuickProtobuf,
    ) -> impl Iterator<Item = crate::Result<Self>> {
      [].into_iter()
    }
  }

  impl<D> Serialize<QuickProtobuf> for ProtobufResponse<D>
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
