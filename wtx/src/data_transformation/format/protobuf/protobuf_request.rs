use crate::{data_transformation::dnsn::Serialize, misc::Vector};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("Protobuf request")]
pub struct ProtobufRequest<D> {
  /// Actual data
  pub data: D,
}

impl<D> Serialize<()> for ProtobufRequest<D> {
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
      format::ProtobufRequest,
    },
    misc::Vector,
  };
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  impl<'de, D> Deserialize<'de, QuickProtobuf> for ProtobufRequest<D>
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

  impl<D> Serialize<QuickProtobuf> for ProtobufRequest<D>
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
