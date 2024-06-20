use crate::client_api_framework::{dnsn::Serialize, Id};
use alloc::vec::Vec;
use core::{
  borrow::Borrow,
  cmp::Ordering,
  hash::{Hash, Hasher},
};

/// A rpc call is represented by sending a [JsonRpcRequest] object to a counterpart.
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

impl<P> Serialize<()> for JsonRpcRequest<P> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl<P> Borrow<Id> for JsonRpcRequest<P> {
  #[inline]
  fn borrow(&self) -> &Id {
    &self.id
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
  use crate::client_api_framework::data_format::JsonRpcRequest;

  impl<P> serde::Serialize for JsonRpcRequest<P>
  where
    P: serde::Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::ser::Serializer,
    {
      use serde::ser::SerializeStruct;
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
  use crate::client_api_framework::{data_format::JsonRpcRequest, dnsn::SerdeJson};
  use alloc::vec::Vec;

  impl<P> crate::client_api_framework::dnsn::Serialize<SerdeJson> for JsonRpcRequest<P>
  where
    P: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      serde_json::to_writer(bytes, self)?;
      Ok(())
    }
  }
}

#[cfg(feature = "simd-json")]
mod simd_json {
  use crate::client_api_framework::{data_format::JsonRpcRequest, dnsn::SimdJson};
  use alloc::vec::Vec;

  impl<P> crate::client_api_framework::dnsn::Serialize<SimdJson> for JsonRpcRequest<P>
  where
    P: serde::Serialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vec<u8>, _: &mut SimdJson) -> crate::Result<()> {
      simd_json::to_writer(bytes, self)?;
      Ok(())
    }
  }
}
