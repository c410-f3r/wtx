use crate::{
  collection::Vector,
  de::{
    Decode, DecodeSeq, Encode,
    format::{De, DecodeWrapper, EncodeWrapper},
  },
};

/// `GraphQL` request/operation, can be a query or a mutation.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug)]
pub struct GraphQlEncoder<ON, Q, V> {
  /// Describes what type of operation you're intending to perform.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub operation_name: Option<ON>,
  /// Describes the desired data to be fetched.
  pub query: Q,
  /// Separated data intended to help queries.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub variables: Option<V>,
}

impl<'de, ON, Q, V> Decode<'de, De<()>> for GraphQlEncoder<ON, Q, V>
where
  Q: Default,
{
  #[inline]
  fn decode(_: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { operation_name: None, query: Q::default(), variables: None })
  }
}

impl<'de, ON, Q, V> DecodeSeq<'de, De<()>> for GraphQlEncoder<ON, Q, V>
where
  Q: Default,
{
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut DecodeWrapper<'de>) -> crate::Result<()> {
    Ok(())
  }
}

impl<ON, Q, V> Encode<De<()>> for GraphQlEncoder<ON, Q, V> {
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::de::{format::SerdeJson, protocol::GraphQlEncoder};
  use serde::Serialize;

  _impl_enc! {
    GraphQlEncoder<ON: Serialize, Q: Serialize, V: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.vector, this)?;
    }
  }
}
