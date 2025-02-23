use crate::{
  data_transformation::{
    Id,
    dnsn::{De, DecodeWrapper, EncodeWrapper},
  },
  misc::{Decode, DecodeSeq, Encode, Lease, Vector},
};
use core::{
  borrow::Borrow,
  cmp::Ordering,
  hash::{Hash, Hasher},
};

/// A rpc call is represented by sending a [`JsonRpcRequest`] object to a counterpart.
///
/// The `jsonrpc` field is not included because it will always be `2.0`.
#[derive(Debug)]
pub struct JsonRpcRequest<P> {
  /// An identifier established by the Client
  pub id: Id,
  /// A String containing the name of the method to be invoked
  pub method: &'static str,
  /// A Structured value that holds the parameter values to be used during the invocation of the method
  pub params: P,
}

impl<'de, P> Decode<'de, De<()>> for JsonRpcRequest<P>
where
  P: Default,
{
  #[inline]
  fn decode(_: &mut (), _: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { id: 0, method: "", params: P::default() })
  }
}

impl<'de, P> DecodeSeq<'de, De<()>> for JsonRpcRequest<P>
where
  P: Default,
{
  #[inline]
  fn decode_seq(_: &mut (), _: &mut Vector<Self>, _: &mut DecodeWrapper<'de>) -> crate::Result<()> {
    Ok(())
  }
}

impl<P> Borrow<Id> for JsonRpcRequest<P> {
  #[inline]
  fn borrow(&self) -> &Id {
    &self.id
  }
}

impl<P> Encode<De<()>> for JsonRpcRequest<P> {
  #[inline]
  fn encode(&self, _: &mut (), _: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}

impl<P> Eq for JsonRpcRequest<P> {}

impl<P> Hash for JsonRpcRequest<P> {
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    self.id.hash(state);
  }
}

impl<P> Lease<Id> for JsonRpcRequest<P> {
  #[inline]
  fn lease(&self) -> &Id {
    &self.id
  }
}

impl<P> Ord for JsonRpcRequest<P> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.id.cmp(&other.id)
  }
}

impl<P> PartialEq for JsonRpcRequest<P> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<P> PartialOrd for JsonRpcRequest<P> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::data_transformation::format::JsonRpcRequest;
  use serde::{
    Serialize,
    ser::{SerializeStruct, Serializer},
  };

  impl<P> Serialize for JsonRpcRequest<P>
  where
    P: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut state = serializer.serialize_struct("JsonRpcRequest", 4)?;
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
  use crate::data_transformation::{dnsn::SerdeJson, format::JsonRpcRequest};
  use serde::Serialize;

  _impl_enc! {
    JsonRpcRequest<R: Serialize>,
    SerdeJson,
    |this, _aux, ew| {
      serde_json::to_writer(&mut *ew.vector, this)?;
    }
  }
}
