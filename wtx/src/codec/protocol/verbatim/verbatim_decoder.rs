use crate::{
  codec::{Decode, DecodeSeq, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::Vector,
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[doc = generic_data_format_doc!("verbatim response")]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct VerbatimDecoder<D> {
  /// Actual data
  pub data: D,
}

impl<D> VerbatimDecoder<D> {
  /// Shortcut
  #[inline]
  pub const fn new(data: D) -> Self {
    Self { data }
  }
}

impl<'de, D, EA> Decode<'de, GenericCodec<&mut (), EA>> for VerbatimDecoder<D>
where
  D: Default,
{
  #[inline]
  fn decode(_: &mut DecodeWrapper<'de, &mut ()>) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }
}

impl<'de, D, EA> DecodeSeq<'de, GenericCodec<&mut (), EA>> for VerbatimDecoder<D>
where
  D: Default,
{
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut DecodeWrapper<'de, &mut ()>) -> crate::Result<()> {
    Ok(())
  }
}

impl<D, DA> Encode<GenericCodec<DA, &mut ()>> for VerbatimDecoder<D> {
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_, &mut ()>) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "quick-protobuf")]
mod quick_protobuf {
  use crate::codec::{CodecError, format::QuickProtobuf, protocol::VerbatimDecoder};
  use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

  _impl_dec! {
    VerbatimDecoder<(D): MessageRead<'de>>,
    QuickProtobuf,
    |_aux, dw| {
      Ok(Self { data: MessageRead::from_reader(&mut BytesReader::from_bytes(dw.bytes), dw.bytes)? })
    }
  }

  _impl_dec_seq! {
    VerbatimDecoder<D: MessageRead<'de>>,
    QuickProtobuf,
    |_aux, _buffer, _dw| {
      Err(CodecError::UnsupportedOperation.into())
    }
  }

  _impl_enc! {
    VerbatimDecoder<D: MessageWrite>,
    QuickProtobuf,
    |this, _aux, ew| {
      this.data.write_message(&mut Writer::new(&mut *ew.buffer))?;
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    codec::{
      format::SerdeJson,
      protocol::{VerbatimDecoder, misc::collect_using_serde_json},
    },
    misc::serde_json_deserialize_from_slice,
  };
  use serde::{Deserialize, Serialize};

  _impl_dec! {
    VerbatimDecoder<(D): Deserialize<'de>>,
    SerdeJson,
    |_aux, dw| {
      serde_json_deserialize_from_slice(dw.bytes)
    }
  }

  _impl_dec_seq! {
    VerbatimDecoder<D: Deserialize<'de>>,
    SerdeJson,
    |_aux, buffer, dw| {
      collect_using_serde_json(buffer, &mut dw.bytes)
    }
  }

  _impl_enc! {
    VerbatimDecoder<D: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.buffer, &this.data)?;
    }
  }
}
