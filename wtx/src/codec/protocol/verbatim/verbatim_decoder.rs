use crate::{
  codec::{Decode, DecodeSeq, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
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

impl<'de, D> Decode<'de, GenericCodec<()>> for VerbatimDecoder<D>
where
  D: Default,
{
  #[inline]
  fn decode(_: &mut GenericDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { data: D::default() })
  }
}

impl<'de, D> DecodeSeq<'de, GenericCodec<()>> for VerbatimDecoder<D>
where
  D: Default,
{
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut GenericDecodeWrapper<'de>) -> crate::Result<()> {
    Ok(())
  }
}

impl<D> Encode<GenericCodec<()>> for VerbatimDecoder<D> {
  #[inline]
  fn encode(&self, _: &mut GenericEncodeWrapper<'_>) -> crate::Result<()> {
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
      this.data.write_message(&mut Writer::new(&mut *ew.vector))?;
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
      serde_json::to_writer(&mut *ew.vector, &this.data)?;
    }
  }
}
