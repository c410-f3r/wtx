/// GraphQL request or operation, can be a query or a mutation.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug)]
pub struct GraphQlRequest<ON, Q, V> {
  /// Describes what type of operation you're intending to perform.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub operation_name: Option<ON>,
  /// Describes the desired data to be fetched.
  pub query: Q,
  /// Separated data intended to help queries.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub variables: Option<V>,
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    client_api_framework::{data_format::GraphQlRequest, dnsn::SerdeJson},
    misc::Vector,
  };

  impl<ON, Q, V> crate::client_api_framework::dnsn::Serialize<SerdeJson> for GraphQlRequest<ON, Q, V>
  where
    ON: serde::Serialize,
    Q: serde::Serialize,
    V: serde::Serialize,
  {
    #[inline]
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SerdeJson) -> crate::Result<()> {
      if size_of::<Self>() == 0 {
        return Ok(());
      }
      serde_json::to_writer(bytes, self)?;
      Ok(())
    }
  }
}

#[cfg(feature = "simd-json")]
mod simd_json {
  use crate::{
    client_api_framework::{data_format::GraphQlRequest, dnsn::SimdJson},
    misc::Vector,
  };

  impl<ON, Q, V> crate::client_api_framework::dnsn::Serialize<SimdJson> for GraphQlRequest<ON, Q, V>
  where
    ON: serde::Serialize,
    Q: serde::Serialize,
    V: serde::Serialize,
  {
    fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut SimdJson) -> crate::Result<()> {
      if size_of::<Self>() == 0 {
        return Ok(());
      }
      simd_json::to_writer(bytes, self)?;
      Ok(())
    }
  }
}
