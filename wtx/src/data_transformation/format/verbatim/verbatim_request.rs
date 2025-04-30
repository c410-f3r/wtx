use crate::{
  collection::Vector,
  data_transformation::dnsn::{De, DecodeWrapper, EncodeWrapper},
  misc::{Decode, DecodeSeq, Encode},
};

/// A wrapper for data types that don't require a special pre-fixed structure.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct VerbatimRequest<D> {
  /// Actual data
  pub data: D,
}

impl<'de, D> Decode<'de, De<()>> for VerbatimRequest<D>
where
  D: Default,
{
  #[inline]
  fn decode(_: &mut (), _: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }
}

impl<'de, D> DecodeSeq<'de, De<()>> for VerbatimRequest<D>
where
  D: Default,
{
  #[inline]
  fn decode_seq(_: &mut (), _: &mut Vector<Self>, _: &mut DecodeWrapper<'de>) -> crate::Result<()> {
    Ok(())
  }
}

impl<D> Encode<De<()>> for VerbatimRequest<D> {
  #[inline]
  fn encode(&self, _: &mut (), _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "borsh")]
mod borsh {
  use crate::data_transformation::{DataTransformationError, dnsn::Borsh, format::VerbatimRequest};
  use borsh::{BorshDeserialize, BorshSerialize};

  _impl_dec! {
    VerbatimRequest<D: BorshDeserialize>,
    Borsh,
    |_aux, dw| {
      Ok(Self { data: D::deserialize(&mut dw.bytes)? })
    }
  }

  _impl_dec_seq! {
    VerbatimRequest<D: BorshDeserialize>,
    Borsh,
    |_aux, _buffer, _dw| {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }

  _impl_enc! {
    VerbatimRequest<D: BorshSerialize>,
    Borsh,
    |this, _aux, ew| {
      this.data.serialize(&mut ew.vector)?;
    }
  }
}

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::data_transformation::{
    DataTransformationError, dnsn::QuickProtobuf, format::VerbatimRequest,
  };
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  _impl_dec! {
    VerbatimRequest<D: MessageRead<'de>>,
    QuickProtobuf,
    |_aux, dw| {
      Ok(Self { data: MessageRead::from_reader(&mut BytesReader::from_bytes(dw.bytes), dw.bytes)? })
    }
  }

  _impl_dec_seq! {
    VerbatimRequest<D: MessageRead<'de>>,
    QuickProtobuf,
    |_aux, _buffer, _dw| {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }

  _impl_enc! {
    VerbatimRequest<D: MessageWrite>,
    QuickProtobuf,
    |this, _aux, ew| {
      this.data.write_message(&mut Writer::new(&mut *ew.vector))?;
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::data_transformation::{
    dnsn::SerdeJson,
    format::{VerbatimRequest, misc::collect_using_serde_json},
  };
  use serde::{Deserialize, Serialize};

  _impl_dec! {
    VerbatimRequest<D: Deserialize<'de>>,
    SerdeJson,
    |_aux, dw| {
      Ok(serde_json::from_slice(dw.bytes)?)
    }
  }

  _impl_dec_seq! {
    VerbatimRequest<D: Deserialize<'de>>,
    SerdeJson,
    |_aux, buffer, dw| {
      collect_using_serde_json(buffer, &mut dw.bytes)
    }
  }

  _impl_enc! {
    VerbatimRequest<D: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.vector, &this.data)?;
    }
  }
}

#[cfg(feature = "serde_urlencoded")]
mod urlencoded {
  use crate::data_transformation::{
    DataTransformationError, dnsn::Urlencoded, format::VerbatimRequest,
  };
  use serde::{Deserialize, Serialize};

  _impl_dec! {
    VerbatimRequest<D: Deserialize<'de>>,
    Urlencoded,
    |_aux, dw| {
      Ok(serde_urlencoded::from_bytes(dw.bytes)?)
    }
  }

  _impl_dec_seq! {
    VerbatimRequest<D: Deserialize<'de>>,
    Urlencoded,
    |_aux, _buffer, _dw| {
      Err(DataTransformationError::UnsupportedOperation.into())
    }
  }

  _impl_enc! {
    VerbatimRequest<D: Serialize>,
    Urlencoded,
    |this, _aux, ew| {
      ew.vector.extend_from_copyable_slice(serde_urlencoded::to_string(&this.data)?.as_bytes())?;
    }
  }
}
