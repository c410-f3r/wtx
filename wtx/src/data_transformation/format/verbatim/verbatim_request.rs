use crate::{
  data_transformation::dnsn::{DecodeWrapper, Dnsn, EncodeWrapper},
  misc::{Decode, DecodeSeq, Encode, Vector},
};

/// A wrapper for data types that don't require a special pre-fixed structure.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct VerbatimRequest<D> {
  /// Actual data
  pub data: D,
}

impl<'de, D> Decode<'de, Dnsn<()>> for VerbatimRequest<D>
where
  D: Default,
{
  #[inline]
  fn decode(_: &mut DecodeWrapper<'_, 'de, ()>) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }
}

impl<'de, D> DecodeSeq<'de, Dnsn<()>> for VerbatimRequest<D>
where
  D: Default,
{
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut DecodeWrapper<'_, 'de, ()>) -> crate::Result<()> {
    Ok(())
  }
}

impl<D> Encode<Dnsn<()>> for VerbatimRequest<D> {
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_, ()>) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::{
    data_transformation::{
      dnsn::{Borsh, DecodeWrapper, Dnsn, EncodeWrapper},
      format::VerbatimRequest,
      DataTransformationError,
    },
    misc::{Decode, DecodeSeq, Encode, Vector},
  };
  use borsh::{BorshDeserialize, BorshSerialize};

  impl<'de, D> Decode<'de, Dnsn<Borsh>> for VerbatimRequest<D>
  where
    D: BorshDeserialize,
  {
    #[inline]
    fn decode(dw: &mut DecodeWrapper<'_, 'de, Borsh>) -> crate::Result<Self> {
      Ok(Self { data: D::deserialize(&mut dw.bytes)? })
    }
  }

  impl<'de, D> DecodeSeq<'de, Dnsn<Borsh>> for VerbatimRequest<D>
  where
    D: BorshDeserialize,
  {
    #[inline]
    fn decode_seq(
      _: &mut Vector<Self>,
      _: &mut DecodeWrapper<'_, 'de, Borsh>,
    ) -> crate::Result<()> {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }

  impl<D> Encode<Dnsn<Borsh>> for VerbatimRequest<D>
  where
    D: BorshSerialize,
  {
    #[inline]
    fn encode(&self, ew: &mut EncodeWrapper<'_, Borsh>) -> crate::Result<()> {
      if size_of::<Self>() == 0 {
        return Ok(());
      }
      self.data.serialize(&mut ew.vector)?;
      Ok(())
    }
  }
}

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::{
    data_transformation::{
      dnsn::{DecodeWrapper, Dnsn, EncodeWrapper, QuickProtobuf},
      format::VerbatimRequest,
      DataTransformationError,
    },
    misc::{Decode, DecodeSeq, Encode, Vector},
  };
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  impl<'de, D> Decode<'de, Dnsn<QuickProtobuf>> for VerbatimRequest<D>
  where
    D: MessageRead<'de>,
  {
    #[inline]
    fn decode(dw: &mut DecodeWrapper<'_, 'de, QuickProtobuf>) -> crate::Result<Self> {
      Ok(Self { data: MessageRead::from_reader(&mut BytesReader::from_bytes(dw.bytes), dw.bytes)? })
    }
  }

  impl<'de, D> DecodeSeq<'de, Dnsn<QuickProtobuf>> for VerbatimRequest<D>
  where
    D: MessageRead<'de>,
  {
    #[inline]
    fn decode_seq(
      _: &mut Vector<Self>,
      _: &mut DecodeWrapper<'_, 'de, QuickProtobuf>,
    ) -> crate::Result<()> {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }

  impl<D> Encode<Dnsn<QuickProtobuf>> for VerbatimRequest<D>
  where
    D: MessageWrite,
  {
    #[inline]
    fn encode(&self, ew: &mut EncodeWrapper<'_, QuickProtobuf>) -> crate::Result<()> {
      if size_of::<Self>() == 0 {
        return Ok(());
      }
      self.data.write_message(&mut Writer::new(&mut *ew.vector))?;
      Ok(())
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    data_transformation::{
      dnsn::{DecodeWrapper, Dnsn, EncodeWrapper, SerdeJson},
      format::{misc::collect_using_serde_json, VerbatimRequest},
    },
    misc::{Decode, DecodeSeq, Encode, Vector},
  };
  use serde::{Deserialize, Serialize};

  impl<'de, D> Decode<'de, Dnsn<SerdeJson>> for VerbatimRequest<D>
  where
    D: Deserialize<'de>,
  {
    #[inline]
    fn decode(dw: &mut DecodeWrapper<'_, 'de, SerdeJson>) -> crate::Result<Self> {
      Ok(serde_json::from_slice(dw.bytes)?)
    }
  }

  impl<'de, D> DecodeSeq<'de, Dnsn<SerdeJson>> for VerbatimRequest<D>
  where
    D: Deserialize<'de>,
  {
    #[inline]
    fn decode_seq(
      buffer: &mut Vector<Self>,
      dw: &mut DecodeWrapper<'_, 'de, SerdeJson>,
    ) -> crate::Result<()> {
      collect_using_serde_json(buffer, &mut dw.bytes)
    }
  }

  impl<D> Encode<Dnsn<SerdeJson>> for VerbatimRequest<D>
  where
    D: Serialize,
  {
    #[inline]
    fn encode(&self, ew: &mut EncodeWrapper<'_, SerdeJson>) -> crate::Result<()> {
      if size_of::<Self>() == 0 {
        return Ok(());
      }
      serde_json::to_writer(&mut *ew.vector, &self.data)?;
      Ok(())
    }
  }
}
