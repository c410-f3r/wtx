use crate::{
  codec::{
    Decode, DecodeSeq, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper, Id,
  },
  collection::Vector,
  misc::Lease,
};
use core::{
  borrow::Borrow,
  cmp::Ordering,
  hash::{Hash, Hasher},
};

/// A rpc call is represented by sending a [`JsonRpcEncoder`] object to a counterpart.
///
/// The `jsonrpc` field is not included because it will always be `2.0`.
#[derive(Debug)]
pub struct JsonRpcEncoder<P> {
  /// An identifier established by the Client
  pub id: Id,
  /// A String containing the name of the method to be invoked
  pub method: &'static str,
  /// A Structured value that holds the parameter values to be used during the invocation of the method
  pub params: P,
}

impl<'de, EA, P> Decode<'de, GenericCodec<(), EA>> for JsonRpcEncoder<P>
where
  P: Default,
{
  #[inline]
  fn decode(_: &mut GenericDecodeWrapper<'de, ()>) -> crate::Result<Self> {
    Ok(Self { id: 0, method: "", params: P::default() })
  }
}

impl<'de, EA, P> DecodeSeq<'de, GenericCodec<(), EA>> for JsonRpcEncoder<P>
where
  P: Default,
{
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut GenericDecodeWrapper<'de, ()>) -> crate::Result<()> {
    Ok(())
  }
}

impl<P> Borrow<Id> for JsonRpcEncoder<P> {
  #[inline]
  fn borrow(&self) -> &Id {
    &self.id
  }
}

impl<DA, P> Encode<GenericCodec<DA, ()>> for JsonRpcEncoder<P> {
  #[inline]
  fn encode(&self, _: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    Ok(())
  }
}

impl<P> Eq for JsonRpcEncoder<P> {}

impl<P> Hash for JsonRpcEncoder<P> {
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    self.id.hash(state);
  }
}

impl<P> Lease<Id> for JsonRpcEncoder<P> {
  #[inline]
  fn lease(&self) -> &Id {
    &self.id
  }
}

impl<P> Ord for JsonRpcEncoder<P> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.id.cmp(&other.id)
  }
}

impl<P> PartialEq for JsonRpcEncoder<P> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<P> PartialOrd for JsonRpcEncoder<P> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::codec::protocol::JsonRpcEncoder;
  use serde::{
    Serialize,
    ser::{SerializeStruct, Serializer},
  };

  impl<P> Serialize for JsonRpcEncoder<P>
  where
    P: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut state = serializer.serialize_struct("JsonRpcEncoder", 4)?;
      state.serialize_field("jsonrpc", "2.0")?;
      state.serialize_field("method", self.method)?;
      if size_of::<P>() > 0 {
        state.serialize_field("params", &self.params)?;
      }
      state.serialize_field("id", &self.id)?;
      state.end()
    }
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::codec::{format::SerdeJson, protocol::JsonRpcEncoder};
  use serde::Serialize;

  _impl_enc! {
    JsonRpcEncoder<R: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.buffer, this)?;
    }
  }
}
