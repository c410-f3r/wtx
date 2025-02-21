use crate::{data_transformation::dnsn::Serialize, misc::Vector};

/// `GraphQL` request/operation, can be a query or a mutation.
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

impl<ON, Q, V> Serialize<()> for GraphQlRequest<ON, Q, V> {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut ()) -> crate::Result<()> {
    Ok(())
  }
}

impl<DRSR, ON, Q, V> Serialize<&mut DRSR> for GraphQlRequest<ON, Q, V>
where
  GraphQlRequest<ON, Q, V>: Serialize<DRSR>,
{
  #[inline]
  fn to_bytes(&mut self, bytes: &mut Vector<u8>, drsr: &mut &mut DRSR) -> crate::Result<()> {
    self.to_bytes(bytes, drsr)
  }
}

#[cfg(feature = "serde_json")]
mod serde_json {
  use crate::{
    data_transformation::{dnsn::SerdeJson, format::GraphQlRequest},
    misc::Vector,
  };
  use serde::Serialize;

  impl<ON, Q, V> crate::data_transformation::dnsn::Serialize<SerdeJson> for GraphQlRequest<ON, Q, V>
  where
    ON: Serialize,
    Q: Serialize,
    V: Serialize,
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
